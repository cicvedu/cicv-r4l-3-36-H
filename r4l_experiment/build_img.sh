export R4L_EXP=$PWD

rm -rf $R4L_EXP/initramfs

mkdir $R4L_EXP/initramfs
cd $R4L_EXP/initramfs

# Create necessary directories
mkdir -p {bin,dev,etc,lib,lib64,mnt,proc,root,sbin,sys,tmp}

# Set Permission
chmod 1777 tmp

# Copy necessary device files from host, root privilege maybe needed.
sudo cp -a /dev/{null,console,tty,ttyS0} dev/

cd $R4L_EXP/initramfs

cp $R4L_EXP/../busybox-1.36.1/busybox  ./bin/
chmod +x bin/busybox
# Install busybox
bin/busybox --install bin
bin/busybox --install sbin

cd $R4L_EXP/initramfs

cat << EOF > init
#!/bin/busybox sh

# Mount the /proc and /sys filesystems.
mount -t proc none /proc
mount -t sysfs none /sys

# Boot real things.

# NIC up
ip link set eth0 up
ip addr add 10.0.2.15/24 dev eth0
ip link set lo up

# Wait for NIC ready
sleep 0.5

mount -t nfs -o nolock host_machine:$R4L_EXP/driver /mnt

# 同样在init脚本中设置自动挂载，在NFS设置后面加入
mount -t devpts devpts  /dev/pts
mknod -m 666 dev/ptmx c 5 2

# 同样在init脚本中设置自动启动，在telnetserver设置后面加入
telnetd -l /bin/sh  -p 23

# Make the new shell as a login shell with -l option
# Only login shell read /etc/profile
setsid sh -c 'exec sh -l </dev/ttyS0 >/dev/ttyS0 2>&1'

EOF

chmod +x init

cd $R4L_EXP/initramfs

# name resolve
cat << EOF > etc/hosts
127.0.0.1    localhost
10.0.2.2     host_machine
EOF

# common alias
cat << EOF > etc/profile
alias ll='ls -l'
EOF

# busybox saves password in /etc/passwd directly, no /etc/shadow is needed.
cat << EOF > etc/passwd
root:x:0:0:root:/root:/bin/bash
EOF

# group file
cat << EOF > etc/group
root:x:0:
EOF

cd $R4L_EXP/initramfs

mkdir dev/pts
# 然后rebuild initramfs
cd $R4L_EXP/initramfs
find . -print0 | cpio --null -ov --format=newc | gzip -9 > ../initramfs.cpio.gz

cd $R4L_EXP

cat << EOF > $R4L_EXP/boot.sh
# 以下是boot.sh的内容：
#!/bin/sh
kernel_image="../linux/arch/x86/boot/bzImage"

qemu-system-x86_64 \
-kernel \$kernel_image \
-append "console=ttyS0 root=/dev/sda rw" \
-initrd ./initramfs.cpio.gz \
-netdev user,id=host_net,hostfwd=tcp::7023-:23 \
-device e1000,mac=52:54:00:12:34:50,netdev=host_net \
-nographic
EOF

# 然后执行以下命令启动
chmod +x boot.sh
./boot.sh 