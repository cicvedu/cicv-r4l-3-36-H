> 实验链接:  [第四期训练营-第三周rust for linux 作业说明 (qq.com)](https://docs.qq.com/doc/DSk5xTHRJY1FZVUdK?u=ae25377a587e4962b883292e9fb566dd)
> 
> 代码链接: [cicvedu/cicv-r4l-3-36-H (github.com)](https://github.com/cicvedu/cicv-r4l-3-36-H)

## 作业1：编译Linux内核

make x86\_64\_defconfig

![6697835f1f284.png](https://img.36h.top/i/2024/07/17/6697835f1f284.png)

make LLVM=1 menuconfig

![669783ae11705.png](https://img.36h.top/i/2024/07/17/669783ae11705.png)
![669783b80efa6.png](https://img.36h.top/i/2024/07/17/669783b80efa6.png)

make LLVM=1 -j\$(nproc)

![669783bf27496.png](https://img.36h.top/i/2024/07/17/669783bf27496.png)

![669783cf18e84.png](https://img.36h.top/i/2024/07/17/669783cf18e84.png)

## 作业2：对Linux内核进行一些配置

**Q:** **在该文件夹中调用make LLVM=1，该文件夹内的代码将编译成一个内核模块。请结合你学到的知识，回答以下两个问题：**

1、         编译成内核模块，是在哪个文件中以哪条语句定义的？

![669783e2bae69.png](https://img.36h.top/i/2024/07/17/669783e2bae69.png)

通过 kbuild 文件中的 obj-m := r4l\_e1000\_demo.o 语句使得内核知道要将其编译为内核模块
2、该模块位于独立的文件夹内，却能编译成Linux内核模块，这叫做out-of-tree module，请分析它是如何与内核代码产生联系的？

![669783ea3f5c5.png](https://img.36h.top/i/2024/07/17/669783ea3f5c5.png)

执行 make 命令时，Makefile 会切换到指定的内核源码目录，并通过 M=\$\$PWD 参数将当前模块目录指定为编译目标。

编译e1000模块

![669783ffa805e.png](https://img.36h.top/i/2024/07/17/669783ffa805e.png)

重新编译Linux内核 禁用c版本的e1000驱动

![6697840ada3cf.png](https://img.36h.top/i/2024/07/17/6697840ada3cf.png)
进入qemu

![669784149cbbd.png](https://img.36h.top/i/2024/07/17/669784149cbbd.png)

手动配置网卡

![6697841d0a603.png](https://img.36h.top/i/2024/07/17/6697841d0a603.png)

Ping 10.0.2.2

![6697842569a7f.png](https://img.36h.top/i/2024/07/17/6697842569a7f.png)

## 作业3：使用rust编写一个简单的内核模块并运行

添加一个rust\_helloworld.rs文件

![669784313ae92.png](https://img.36h.top/i/2024/07/17/669784313ae92.png)

修改samples/rust下的Makefile和Kconfig

![6697843dce930.png](https://img.36h.top/i/2024/07/17/6697843dce930.png)

![66978445a1178.png](https://img.36h.top/i/2024/07/17/66978445a1178.png)

make LLVM=1 menuconfig

![6697844b7692c.png](https://img.36h.top/i/2024/07/17/6697844b7692c.png)

测试模块

![66978452594ce.png](https://img.36h.top/i/2024/07/17/66978452594ce.png)
![6697845a4c80e.png](https://img.36h.top/i/2024/07/17/6697845a4c80e.png)

![](file:///E:/TMP/msohtmlclip1/01/clip_image028.png)

## 作业4：为e1000网卡驱动添加remove代码

分析 e1000注册/移除流程

![6697846505162.png](https://img.36h.top/i/2024/07/17/6697846505162.png)

![6697846986a56.png](https://img.36h.top/i/2024/07/17/6697846986a56.png)

实现代码：

![6697846fc1a08.png](https://img.36h.top/i/2024/07/17/6697846fc1a08.png)

![6697847fafefa.png](https://img.36h.top/i/2024/07/17/6697847fafefa.png)

![669784850676b.png](https://img.36h.top/i/2024/07/17/669784850676b.png)

移除模块并重新插入

![6697848c6300b.png](https://img.36h.top/i/2024/07/17/6697848c6300b.png)

再次测试ping 符合预期输出

![6697849561bcc.png](https://img.36h.top/i/2024/07/17/6697849561bcc.png)



## 作业5：注册字符设备

修改rust\_chdevs的实现

![6697849d6eb76.png](https://img.36h.top/i/2024/07/17/6697849d6eb76.png)



执行测试命令

![669784a326102.png](https://img.36h.top/i/2024/07/17/669784a326102.png)



## 实战试验

![669784ad09e53.png](https://img.36h.top/i/2024/07/17/669784ad09e53.png)