# Kitties Course

* [Substrate Node README](node/README.md)
* [Front-end README](frontend/README.md)

To make polkadot-JS app connects to the Substrate node, add 
[`frontend/src/config/types.json`](frontend/src/config/types.json)
JSON structure in app `Settings` > `Developer`.


## 课后作业完成情况

1、完成创建一个毛孩

2、完成展示毛孩为卡片

3、未完成转让毛孩操作

## 补充说明
1、毛孩图片会在Block Finalized阶段展示出来，每次都会重新渲染，会有卡顿；且UI没有经过优化；

2、如果kittyindex为0的毛孩没有展示出来，需要再次点击创建；

3、切换账号时，每个账号只会展示自己的毛孩，不会展示其他人的；

4、目前前端开发能力有限，暂时没有在前端完成转让操作；
