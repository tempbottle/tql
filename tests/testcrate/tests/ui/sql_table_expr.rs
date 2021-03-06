/*
 * Copyright (c) 2017-2018 Boucher, Antoni <bouanto@zoho.com>
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy of
 * this software and associated documentation files (the "Software"), to deal in
 * the Software without restriction, including without limitation the rights to
 * use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
 * the Software, and to permit persons to whom the Software is furnished to do so,
 * subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
 * FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
 * COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
 * IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
 * CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
 */

//! Tests of the type analyzer lint for the `#[SqlTable]` attribute.

#![feature(proc_macro)]

extern crate tql;
#[macro_use]
extern crate tql_macros;

#[macro_use]
mod connection;
backend_extern_crate!();

use tql::{ForeignKey, PrimaryKey};

struct Connection {
    value: String,
}

#[derive(SqlTable)]
struct Table {
    //~^ WARNING No primary key found
    field1: String,
    related_field1: ForeignKey<Connection>,
    //~^ the trait bound `Connection: tql::SqlTable` is not satisfied
    // FIXME: when we can check types in proc-macro, output the following.
    // ~^ ERROR `Connection` does not name an SQL table
    // ~| HELP did you forget to add the #[derive(SqlTable)] attribute on the Connection struct?
    related_field2: ForeignKey<RelatedTable>,
}

#[derive(SqlTable)]
struct RelatedTable {
    id: PrimaryKey,
}

fn main() {
}
