// SPDX-License-Identifier: GPL-2.0

//! Character devices.
//!
//! Also called "char devices", `chrdev`, `cdev`.
//!
//! C header: [`include/linux/cdev.h`](../../../../include/linux/cdev.h)
//!
//! Reference: <https://www.kernel.org/doc/html/latest/core-api/kernel-api.html#char-devices>

use alloc::boxed::Box;
use core::convert::TryInto;
use core::marker::PhantomPinned;
use core::pin::Pin;

use crate::{bindings, pr_err};
use crate::error::{code::*, Error, Result};
use crate::file;
use crate::str::CStr;
use crate::Opaque;
// use core::ffi::c_void;

/// Character device.
///
/// # Invariants
///
///   - [`self.0`] is valid and non-null.
///   - [`(*self.0).ops`] is valid, non-null and has static lifetime.
///   - [`(*self.0).owner`] is valid and, if non-null, has module lifetime.
struct Cdev(*mut bindings::cdev);

impl Cdev {
    fn alloc(
        fops: &'static bindings::file_operations,
        module: &'static crate::ThisModule,
    ) -> Result<Self> {
        // SAFETY: FFI call.
        let cdev = unsafe { bindings::cdev_alloc() };
        if cdev.is_null() {
            return Err(ENOMEM);
        }
        // SAFETY: `cdev` is valid and non-null since `cdev_alloc()`
        // returned a valid pointer which was null-checked.
        unsafe {
            (*cdev).ops = fops;
            (*cdev).owner = module.0;
        }
        // INVARIANTS:
        //   - [`self.0`] is valid and non-null.
        //   - [`(*self.0).ops`] is valid, non-null and has static lifetime,
        //     because it was coerced from a reference with static lifetime.
        //   - [`(*self.0).owner`] is valid and, if non-null, has module lifetime,
        //     guaranteed by the [`ThisModule`] invariant.
        Ok(Self(cdev))
    }

    fn add(&mut self, dev: bindings::dev_t, count: core::ffi::c_uint) -> Result {
        // SAFETY: According to the type invariants:
        //   - [`self.0`] can be safely passed to [`bindings::cdev_add`].
        //   - [`(*self.0).ops`] will live at least as long as [`self.0`].
        //   - [`(*self.0).owner`] will live at least as long as the
        //     module, which is an implicit requirement.
        let rc = unsafe { bindings::cdev_add(self.0, dev, count) };
        if rc != 0 {
            return Err(Error::from_kernel_errno(rc));
        }
        Ok(())
    }
}

impl Drop for Cdev {
    fn drop(&mut self) {
        // SAFETY: [`self.0`] is valid and non-null by the type invariants.
        unsafe {
            bindings::cdev_del(self.0);
        }
    }
}

/// completion device registration.
///
pub struct Registration<const N: usize> {
    name: &'static CStr,
    minors_start: u16,
    dev: bindings::dev_t,
    /// completion dev
    pub completion: Opaque<bindings::completion>,
    cdev: Option<Cdev>,
    this_module: &'static crate::ThisModule,
    _pin: PhantomPinned,
}

impl<const N: usize> Registration<{ N }> {
    /// Creates a [`Registration`] object for a character device.
    ///
    /// This does *not* register the device: see [`Self::register()`].
    ///
    /// This associated function is intended to be used when you need to avoid
    /// a memory allocation, e.g. when the [`Registration`] is a member of
    /// a bigger structure inside your [`crate::Module`] instance. If you
    /// are going to pin the registration right away, call
    /// [`Self::new_pinned()`] instead.
    pub fn new(
        name: &'static CStr,
        minors_start: u16,
        this_module: &'static crate::ThisModule,
    ) -> Self {
        let completion = Opaque::uninit();
        Registration {
            name,
            minors_start,
            completion,
            this_module,
            cdev: None,
            _pin: PhantomPinned,
            dev: 0,
        }
    }

    /// Creates a pinned [`Registration`] object for a character device.
    ///
    /// This does *not* register the device: see [`Self::register()`].
    pub fn new_pinned(
        name: &'static CStr,
        minors_start: u16,
        this_module: &'static crate::ThisModule,
    ) -> Result<Pin<Box<Self>>> {
        Ok(Pin::from(Box::try_new(Self::new(
            name,
            minors_start,
            this_module,
        ))?))
    }

    /// Registers a character device.
    ///
    /// You may call this once per device type, up to `N` times.
    pub fn register<T: file::Operations<OpenData = ()>>(
        self: Pin<&mut Self>,
    ) -> Result {
        // SAFETY: We must ensure that we never move out of `this`.
        let this = unsafe { self.get_unchecked_mut() };
        unsafe { bindings::init_completion(this.completion.get());
        };
        pr_err!("register: this completion addr is {:#?}\n",this.completion.get());
        // SAFETY: Calling unsafe function. `this.name` has `'static`
        // lifetime.
        let res = unsafe {
            bindings::alloc_chrdev_region(
                &mut this.dev,
                this.minors_start.into(),
                N.try_into()?,
                this.name.as_char_ptr(),
            )
        };
        if res != 0 {
            return Err(Error::from_kernel_errno(res));
        }
        // SAFETY: The adapter doesn't retrieve any state yet, so it's compatible with any
        // registration.
        let fops = unsafe { file::OperationsVtable::<Self, T>::build() };
        let mut cdev = Cdev::alloc(fops, this.this_module)?;
        cdev.add(this.dev, 1)?;
        this.cdev = Some(cdev);
        Ok(())
    }
}

impl<const N: usize> file::OpenAdapter<()> for Registration<{ N }> {
    unsafe fn convert(_inode: *mut bindings::inode, _file: *mut bindings::file) -> *const () {
        // let reg = crate::container_of!(unsafe { (*inode).__bindgen_anon_4 }, Self, cdev.unwrap().0);
        // unsafe {
        //     (*(file)).private_data = (*reg).completion.get() as *mut c_void;
        // }
        // pr_err!("convert: this completion addr is {:#?}\n",(*reg).completion.get());
        &()
    }
}

// SAFETY: `Registration` does not expose any of its state across threads
// (it is fine for multiple threads to have a shared reference to it).
unsafe impl<const N: usize> Sync for Registration<{ N }> {}

impl<const N: usize> Drop for Registration<{ N }> {
    fn drop(&mut self) {
        // SAFETY: [`self.inner`] is Some, so [`inner.dev`] was previously
        // created using [`bindings::alloc_chrdev_region`].
        unsafe {
            bindings::unregister_chrdev_region(self.dev, N.try_into().unwrap());
        }
    }
}
