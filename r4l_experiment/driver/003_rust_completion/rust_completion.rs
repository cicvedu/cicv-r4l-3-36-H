// SPDX-License-Identifier: GPL-2.0

//! Example of Kernel's completion mechanism
use core::result::Result::Ok;
use kernel::completion;
use kernel::prelude::*;
use kernel::{bindings, file, ThisModule};
// use kernel::Opaque;
// use core::marker::PhantomPinned;
use kernel::task::Task;
use alloc::boxed::Box;
use kernel::pr_info;
use kernel::macros::module;
use kernel::io_buffer::IoBufferWriter;
use kernel::io_buffer::IoBufferReader;


module! {
    type: RustCompletion,
    name: "rust_completion",
    author: "36-H",
    description: "Example of Kernel's completion mechanism",
    license: "GPL",
}

struct CompletionDev {
    // cdev: Cdev,
    completion: *mut bindings::completion,
    // _pin: PhantomPinned,
}
// static COMPLETION_DEV:Arc<CompletionDev> = Arc::try_new(CompletionDev {
//     completion:Opaque::uninit(),
//     _pin:PhantomPinned
// }).unwrap();
// static COMPLETION_MAJOR: u16 = 0;
// static COMPLETION_MINOR: u16 = 0;
static mut COMPLETION_DEV:CompletionDev = CompletionDev{
    completion: 0 as *mut kernel::bindings::completion,
};

unsafe impl Sync for CompletionDev{}
unsafe impl Send for CompletionDev{}

#[vtable]
impl file::Operations for CompletionDev {
    type Data = Box<Self>;

    fn open(_shared: &(), _file: &file::File) -> Result<Box<Self>> {
        pr_info!("open() is invoked\n");
        // let mut _completion = Box::try_new(CompletionDev {
        //     completion:Opaque::uninit(),
        //     _pin:PhantomPinned
        // })?;
        // // let completion = Arc::clone(&COMPLETION_DEV);
        // unsafe {
        //     bindings::init_completion(_completion.completion.get());
        // };
        // unsafe{
            // let completion: *mut bindings::completion = (*(file.get_inner().get())).private_data as *mut bindings::completion;
            
        // }
        Ok(Box::try_new(
            CompletionDev {
                // cdev: Cdev,
                completion: unsafe{COMPLETION_DEV.completion},
                // _pin: PhantomPinned,
            }
        )?)
    }

    fn read(
        _this: &Self,
        _file: &file::File,
        _writer: &mut impl IoBufferWriter,
        _offset: u64,
    ) -> Result<usize> {
        pr_info!("read() is invoked\n");
        

        let task = Task::current();

        pr_info!(
            "process {} is going to sleep\n",
            task.pid()
        );
        
        unsafe { bindings::wait_for_completion(_this.completion) };
        // unsafe { bindings::wait_for_completion((*(_file.get_inner().get())).private_data as *mut bindings::completion) };

        pr_info!(
            "process {} is awoken\n",
            task.pid()
        );
        Ok(0)
    }

    fn write(
        _this: &Self,
        _file: &file::File,
        _reader: &mut impl IoBufferReader,
        _offset: u64,
    ) -> Result<usize> {
        pr_info!("write() is invoked\n");
        let task = Task::current();

        pr_info!(
            "process {} awakening the readers...\n",
            task.pid()
        );
        unsafe { bindings::complete(_this.completion) };
        // unsafe { bindings::complete((*(_file.get_inner().get())).private_data as *mut bindings::completion as *mut bindings::completion) };
        pr_info!("data.len is {}\n",_reader.len());
        Ok(_reader.len())
    }

    fn release(_data: Self::Data, _file: &file::File) {
        pr_info!("release() is invoked\n");
    }
}

struct RustCompletion {
    _reg: Pin<Box<completion::Registration<1>>>,
}

impl kernel::Module for RustCompletion {
    // 目的实现
    //int err = 0;
    // dev_t devno;
    // printk(KERN_WARNING MODULE_NAME " is loaded\n");
    // // 初始化完成变量
    // init_completion(&completion_dev.completion);
    // // 分配字符设备区域
    // err = alloc_chrdev_region(&devno, completion_minor, 1, MODULE_NAME);
    // if (err < 0) {
    // 	pr_info("Cant't get major");
    // 	return err;
    // }
    // completion_major = MAJOR(devno);
    // // 初始化字符设备
    // cdev_init(&completion_dev.cdev, &completion_fops);
    // // 添加字符设备
    // devno = MKDEV(completion_major, completion_minor); //通过主次设备号来生成dev_t
    // err = cdev_add(&completion_dev.cdev, devno, 1);
    // if (err) {
    // 	pr_info("Error(%d): Adding completion device error\n", err);
    // 	return err;
    // }
    fn init(name: &'static CStr, module: &'static ThisModule) -> Result<Self> {
        pr_info!("completion_example is loaded\n");
        let mut reg = completion::Registration::new_pinned(name, 0, module)?;
        reg.as_mut().register::<CompletionDev>()?;
        unsafe{
            COMPLETION_DEV.completion = reg.completion.get();
        }
        Ok(RustCompletion { _reg:reg })
    }
}

impl Drop for RustCompletion {
    // 目的实现
    // dev_t devno;
    // printk(KERN_WARNING MODULE_NAME " unloaded\n");
    // // 删除字符设备
    // cdev_del(&completion_dev.cdev);
    // // 注销字符设备区域
    // devno = MKDEV(completion_major, completion_minor);
    // unregister_chrdev_region(devno, 1);
    fn drop(&mut self) {
        pr_info!("completion_example is unloaded\n");
    }
}

