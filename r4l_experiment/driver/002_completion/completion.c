#include <linux/module.h>
#include <linux/init.h> 
#include <linux/sched.h>
#include <linux/kernel.h>
#include <linux/fs.h>
#include <linux/cdev.h>
#include <linux/types.h>
#include <linux/completion.h>

#include "completion.h"


static int completion_major = 0, completion_minor = 0;

static struct completion_dev completion_dev;

// 打开设备文件时调用
static
int completion_open(struct inode *inode, struct file *filp)
{
	pr_info("%s() is invoked\n", __FUNCTION__);

	// 将设备结构体保存到文件的私有数据中
	filp->private_data = container_of(inode->i_cdev,
					  struct completion_dev, cdev);

	return 0;
}

// 从设备文件读取时调用
static
ssize_t completion_read(struct file *filp, char __user *buf, size_t count, loff_t *pos)
{
	struct completion_dev *dev = filp->private_data;

	pr_info("%s() is invoked\n", __FUNCTION__);

	pr_info("process %d(%s) is going to sleep\n", current->pid, current->comm);
	// 进程进入睡眠状态，等待完成事件
	wait_for_completion(&dev->completion);
	pr_info("awoken %d(%s)\n", current->pid, current->comm);

	return 0;
}

// 向设备文件写入时调用
static
ssize_t completion_write(struct file *filp, const char __user *buf, size_t count,
		       loff_t *pos)
{
	struct completion_dev *dev = filp->private_data;

	pr_info("%s() is invoked\n", __FUNCTION__);

	pr_info("process %d(%s) awakening the readers...\n",
	       current->pid, current->comm);
	// 完成事件，唤醒所有等待的进程
	complete(&dev->completion);

	return count;
}

// 文件操作结构体
static struct file_operations completion_fops = {
	.owner = THIS_MODULE,
	.open  = completion_open, 
	.read  = completion_read,
	.write = completion_write,
};

// 模块初始化函数
static
int __init m_init(void)
{
	int err = 0;
	dev_t devno;

	printk(KERN_WARNING MODULE_NAME " is loaded\n");

	// 初始化完成变量
	init_completion(&completion_dev.completion);
	// 分配字符设备区域
	err = alloc_chrdev_region(&devno, completion_minor, 1, MODULE_NAME);
	if (err < 0) {
		pr_info("Cant't get major");
		return err;
	}
	completion_major = MAJOR(devno);
	// 初始化字符设备
	cdev_init(&completion_dev.cdev, &completion_fops);
	// 添加字符设备
	devno = MKDEV(completion_major, completion_minor); //通过主次设备号来生成dev_t
	err = cdev_add(&completion_dev.cdev, devno, 1);
	if (err) {
		pr_info("Error(%d): Adding completion device error\n", err);
		return err;
	}

	return 0;
}

// 模块清理函数
static
void __exit m_exit(void)
{
	dev_t devno;

	printk(KERN_WARNING MODULE_NAME " unloaded\n");
	// 删除字符设备
	cdev_del(&completion_dev.cdev);
	// 注销字符设备区域
	devno = MKDEV(completion_major, completion_minor);
	unregister_chrdev_region(devno, 1);
}

module_init(m_init);
module_exit(m_exit);

MODULE_LICENSE("GPL");
MODULE_AUTHOR("Tester");
MODULE_DESCRIPTION("Example of Kernel's completion mechanism");