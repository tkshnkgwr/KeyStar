#![allow(non_snake_case)]

//! # KeyStar
//!
//! 名作「CraftLaunch 2」の原点思想（1行・爆速・キーボード特化・登録ショートカット即起動）を受け継ぐ、
//! Rust製の超軽量キーボードランチャーのコアライブラリ。

pub mod calc;
pub mod clipboard;
pub mod hotkey;
pub mod indexer;
pub mod search;
pub mod single_instance;
pub mod tray;
pub mod ui;
