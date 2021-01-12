### substrate-lessons2
完成课后作业

1、视频中kitties的bug，为transfer方法没有验证owner关系，fix code参考：https://github.com/MrPai/substrate-lessons/blob/main/lesson2/pallets/kitties/src/lib.rs#L147

2、kittyindex在runtime中指定：https://github.com/MrPai/substrate-lessons/blob/main/lesson2/runtime/src/lib.rs#L72

3、获得一个账号所有kitties的存储：https://github.com/MrPai/substrate-lessons/blob/main/lesson2/pallets/kitties/src/lib.rs#L62

4、获得世代关系的数据结构：https://github.com/MrPai/substrate-lessons/blob/main/lesson2/pallets/kitties/src/lib.rs#L55

5、测试代码检查event和error，详见test文件：https://github.com/MrPai/substrate-lessons/blob/main/lesson2/pallets/kitties/src/tests.rs

6、质押token的数据结构：https://github.com/MrPai/substrate-lessons/blob/main/lesson2/pallets/kitties/src/lib.rs#L65

