## 讲解

加入了 runtime， 虽然runtime 中的 loop 看着没进入几次

![alt text](image.png)

主要是因为在这里进行了阻塞，阻塞了 runtime 的 loop，功能类似于线程 sleep，不过更高效

![alt text](image-1.png)

这里还有问题

![alt text](image-2.png)

目前是串行执行的，所以 token唯一 不会出现问题。