# Copilot Instructions for Moetran Native

<!-- markdownlint-disable MD033 -->

## 注释部分

- 所有注释均使用中文。

- 需要的是备忘录式的注释，而不是教学式的注释。

## Vue 3 前端部分

### script 部分

- 使用 TypeScript，即使用 **<script setup lang="ts">**。

- 使用组合式 API。

- 每个核心函数使用 “//”（可多行）进行注释。

- 除非一个大括号作用块内只有一行或两行代码，否则任意两个语句之间都需要添加空格（除非是强相关的几个变量的声明）。

  例如：

  ```ts
  // 单行的例子
  function singleLine() {
    console.log('Single line');
  }

  // 双行的例子
  function doubleLines() {
    console.log('Double lines');
    return 'Ok';
  }

  // 多行的例子（要添加空行）
  function multiLines() {
    console.log('Mulitple lines');

    let result = await someFunc();

    return result;
  }
  ```

- 使用带句尾分号的风格。

- 打日志时，首字母大写，但是不添加 “.” 句号。有错误或上下文时添加进日志。

- 严格遵循 TypeScript 语法，不允许任何的 any，必须使用 interface 进行指名。

- 组件、页面需要实现高内聚低耦合，如子组件所需要的状态从父页面注入。

- 注释流程时，不要给注释中添加流程序号，因为流程很可能更改。

- 一个模拟 mock 函数，必须以 \_\_mock 开头。

### template 和 style 部分

- 暂时不使用 Tailwind CSS 等三方样式库。

- 整体风格简约、现代。

- 无须适配暗色模式，只需要亮色模式。

- 可以使用天蓝色等让人感觉轻松的颜色进行配色，颜色要求统一、清淡。

- style 文件放在 script、template 块之后。

- 要求简洁，减少文字，优先使用图标，能让人一看自明。

- 严格控制布局方式，杜绝 **一切可能产生 scroll bar 的行为**，如组件的高度超出 window 高度等（通过自适应填充或者动态计算宽高及定位起点等方式）。

## Rust 后端部分

- 除非一个大括号作用块内只有一行代码，否则任意两个语句之间都需要添加空格（除非是强相关的几个变量的声明）。

  例如：

  ```rust
  // 单行的例子
  pub async fn single_line() {
      some_func().await;
  }

  // 双行的例子（要添加空行）
  fn double_lines() -> i32 {
      let some_val = func();

      return some_val;
  }

  // 多行的例子（要添加空行）
  fn multi_lines() -> anyhow::Result<()> {
      let prev_index = foo();
      let prev_term = bar();

      let result = calc(prev_index, prev_term)?;

      Ok(result)
  }
  ```

- 对于 tauri::command 指定的函数，必须要使用结构化的 tracing::info! 标识各个函数的执行流。

- 在书写带有 DTO 定义的函数时，不要先把所有 DTO 写出再写 fn，而是将 DTO 与 fn 分组，按一类 DTO + fn 的周期从上到下依次编写。

- 在一些重要或过长的函数中，需要使用 tracing::debug! 来书写报错日志。但是注意，debug 日志可以不用是结构化的，而是语义化的。

- 注释使用中文，但是日志使用英文。
