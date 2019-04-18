# Changelog

## 0.4.0 (2019-04-18)

* Replace `static mut` with safer alternatives.
* Require Rust 1.27.

## 0.3.1 (2018-04-03)

* Minor performance improvement using `Cow` instead of `String` when possible.

## 0.3.0 (2018-02-19)

* Require GTK+ 3.18 and Rust 1.24.

## 0.2.3 (2018-02-19)

* Fix warnings on Rust 1.24.

## 0.2.1 (2016-11-20)

* Convenience functions: `Column::new()`, `PropertyPage::new()`, `FileInfo.add_attribute()`.
* `Menu` and `MenuItem` functions prefer borrowing rather than moving.

## 0.2.0 (2016-11-19)

* Added PropertyPageProvider.

## 0.1.2 (2016-11-16)

* Removed unnecessary unsafe code and argument mutability.

## 0.1.1 (2016-11-12)

* Fixed unnecessary `FileInfo` lifetime and argument mutability.

## 0.1.0 (2016-11-10)

* First public release.
* ColumnProvider, InfoProvider, and MenuProvider are implemented.
* Enough code to allow [tmsu-nautilus-rs](https://github.com/talklittle/tmsu-nautilus-rs) to work.
