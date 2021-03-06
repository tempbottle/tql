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

#![feature(proc_macro, proc_macro_non_items)]

macro_rules! let_vec {
    ( $($name:ident),* = $vector:ident ) => {
        $(let $name = $vector.remove(0);)*
    };
}

use std::str::FromStr;

extern crate chrono;
extern crate tql;
#[macro_use]
extern crate tql_macros;

#[macro_use]
mod connection;
mod teardown;

backend_extern_crate!();

use chrono::DateTime;
use chrono::offset::Utc;
use tql::{ForeignKey, PrimaryKey};
use tql_macros::sql;

use connection::{get_connection, is_not_found};
use teardown::TearDown;

#[derive(SqlTable)]
struct TableSelectExpr {
    id: PrimaryKey,
    field1: String,
    field2: i32,
    related_field: ForeignKey<RelatedTableSelectExpr>,
    optional_field: Option<i32>,
    datetime: DateTime<Utc>,
}

#[derive(SqlTable)]
struct RelatedTableSelectExpr {
    id: PrimaryKey,
    field1: i32,
}

#[derive(SqlTable)]
struct Table1 {
    id: PrimaryKey,
    field1: i32,
    field2: i32,
    related1: ForeignKey<Table2>,
    related2: ForeignKey<Table3>,
    related3: ForeignKey<Table4>,
}

#[derive(SqlTable)]
struct Table2 {
    id: PrimaryKey,
    field1: i32,
    field2: i32,
}

#[derive(SqlTable)]
struct Table3 {
    id: PrimaryKey,
    field1: i32,
    field2: i32,
}

#[derive(SqlTable)]
struct Table4 {
    id: PrimaryKey,
    field1: i32,
    field2: i32,
}

#[derive(SqlTable)]
struct Table5 {
    id: PrimaryKey,
    field1: i32,
    field2: i32,
    //tables6: ManyToMany<Table6>,
}

#[derive(SqlTable)]
struct Table6 {
    id: PrimaryKey,
    field1: i32,
    field2: i32,
    //tables5: HasMany<Table5>,
}

#[derive(SqlTable)]
struct Table5_Table6 {
    table5: ForeignKey<Table5>,
    table6: ForeignKey<Table6>,
}

#[test]
fn test_select() {
    let connection = get_connection();

    let _teardown = TearDown::new(|| {
        let _ = sql!(TableSelectExpr.drop());
        let _ = sql!(RelatedTableSelectExpr.drop());
        let _ = sql!(Table1.drop());
        let _ = sql!(Table2.drop());
        let _ = sql!(Table3.drop());
        let _ = sql!(Table4.drop());
        let _ = sql!(Table5.drop());
        let _ = sql!(Table6.drop());
        let _ = sql!(Table5_Table6.drop());
    });

    let _ = sql!(RelatedTableSelectExpr.create());
    let _ = sql!(TableSelectExpr.create());
    let _ = sql!(Table2.create());
    let _ = sql!(Table3.create());
    let _ = sql!(Table4.create());
    let _ = sql!(Table5.create());
    let _ = sql!(Table6.create());
    let _ = sql!(Table5_Table6.create());
    let _ = sql!(Table1.create());

    let datetime: DateTime<Utc> = FromStr::from_str("2015-11-16T15:51:12-05:00").unwrap();
    let datetime2: DateTime<Utc> = FromStr::from_str("2013-11-15T15:51:12-05:00").unwrap();

    let id = sql!(RelatedTableSelectExpr.insert(field1 = 42)).unwrap();
    let related_field = sql!(RelatedTableSelectExpr.get(id)).unwrap();
    let id = sql!(RelatedTableSelectExpr.insert(field1 = 24)).unwrap();
    let related_field2 = sql!(RelatedTableSelectExpr.get(id)).unwrap();
    let id1 = sql!(TableSelectExpr.insert(
        field1 = "value1",
        field2 = 55,
        related_field = related_field,
        datetime = datetime2,
    )).unwrap();
    let new_field2 = 42;
    let id2 = sql!(TableSelectExpr.insert(field1 = "value2", field2 = new_field2, related_field = related_field,
                                          datetime = datetime2)).unwrap();
    let id3 = sql!(TableSelectExpr.insert(field1 = "value3", field2 = 12, related_field = related_field2,
                                          datetime = datetime2)).unwrap();
    let id4 = sql!(TableSelectExpr.insert(field1 = "value4", field2 = 22, related_field = related_field2,
                                          optional_field = Some(42), datetime = datetime)).unwrap();
    let id5 = sql!(TableSelectExpr.insert(field1 = "value5", field2 = 134, related_field = related_field2,
                                          datetime = datetime2)).unwrap();

    let mut tables = sql!(TableSelectExpr.all()).unwrap();
    assert_eq!(5, tables.len());
    let_vec!(table1, table2, table3, table4, table5 = tables);
    assert_eq!(id1, table1.id);
    assert_eq!("value1", table1.field1);
    assert_eq!(55, table1.field2);
    assert_eq!(id2, table2.id);
    assert_eq!("value2", table2.field1);
    assert_eq!(42, table2.field2);
    assert_eq!(id3, table3.id);
    assert_eq!("value3", table3.field1);
    assert_eq!(12, table3.field2);
    assert_eq!(id4, table4.id);
    assert_eq!("value4", table4.field1);
    assert_eq!(22, table4.field2);
    assert_eq!(id5, table5.id);
    assert_eq!("value5", table5.field1);
    assert_eq!(134, table5.field2);

    let mut tables = sql!(TableSelectExpr.filter(field1 == "value1")).unwrap();
    assert_eq!(1, tables.len());
    let_vec!(table1 = tables);
    assert_eq!("value1", table1.field1);
    assert_eq!(55, table1.field2);

    let mut tables = sql!(TableSelectExpr.filter(field2 >= 42 || field1 == "te'\"\\st")).unwrap();
    assert_eq!(3, tables.len());
    let_vec!(table1, table2, table3 = tables);
    assert_eq!("value1", table1.field1);
    assert_eq!(55, table1.field2);
    assert_eq!("value2", table2.field1);
    assert_eq!(42, table2.field2);
    assert_eq!("value5", table3.field1);
    assert_eq!(134, table3.field2);

    let value = 42;
    let mut tables = sql!(TableSelectExpr.filter(field2 == value)).unwrap();
    assert_eq!(1, tables.len());
    let_vec!(table1 = tables);
    assert_eq!("value2", table1.field1);
    assert_eq!(42, table1.field2);

    let mut tables = sql!(TableSelectExpr.filter(field2 > value)).unwrap();
    assert_eq!(2, tables.len());
    let_vec!(table1, table2 = tables);
    assert_eq!("value1", table1.field1);
    assert_eq!(55, table1.field2);
    assert_eq!("value5", table2.field1);
    assert_eq!(134, table2.field2);

    let value2 = "value1";
    let mut tables = sql!(TableSelectExpr.filter(field2 > value && field1 == value2)).unwrap();
    assert_eq!(1, tables.len());
    let_vec!(table1 = tables);
    assert_eq!("value1", table1.field1);
    assert_eq!(55, table1.field2);

    let value2 = "value2";
    let tables = sql!(TableSelectExpr.filter(field2 > value && field1 == value2)).unwrap();
    assert_eq!(0, tables.len());

    let mut tables = sql!(TableSelectExpr.filter(related_field == related_field)).unwrap();
    assert_eq!(2, tables.len());
    let_vec!(table1, table2 = tables);
    assert_eq!(id1, table1.id);
    assert_eq!(id2, table2.id);

    let mut tables = sql!(TableSelectExpr.filter(related_field == related_field2)).unwrap();
    assert_eq!(3, tables.len());
    let_vec!(table1, table2, table3 = tables);
    assert_eq!(id3, table1.id);
    assert_eq!(id4, table2.id);
    assert_eq!(id5, table3.id);

    let mut tables = sql!(TableSelectExpr.filter(field1 == "value2" || field2 < 100 && field1 == "value1")).unwrap();
    assert_eq!(2, tables.len());
    let_vec!(table1, table2 = tables);
    assert_eq!(id1, table1.id);
    assert_eq!(id2, table2.id);

    let mut tables = sql!(TableSelectExpr.filter((field1 == "value2" || field2 < 100) && field1 == "value1")).unwrap();
    assert_eq!(1, tables.len());
    let_vec!(table1 = tables);
    assert_eq!(id1, table1.id);

    let mut tables = sql!(TableSelectExpr.filter((field1 == "value3" && field2 == 12))).unwrap();
    assert_eq!(1, tables.len());
    let_vec!(table1 = tables);
    assert_eq!(id3, table1.id);

    let mut tables = sql!(TableSelectExpr.filter(!(field1 == "value3" && field2 == 12))).unwrap();
    assert_eq!(4, tables.len());
    let_vec!(table1, table2, table3, table4 = tables);
    assert_eq!(id1, table1.id);
    assert_eq!(id2, table2.id);
    assert_eq!(id4, table3.id);
    assert_eq!(id5, table4.id);

    let mut tables = sql!(TableSelectExpr.filter(!(field2 < 24))).unwrap();
    assert_eq!(3, tables.len());
    let_vec!(table1, table2, table3 = tables);
    assert_eq!(id1, table1.id);
    assert_eq!(id2, table2.id);
    assert_eq!(id5, table3.id);

    let mut tables = sql!(TableSelectExpr.filter(optional_field.is_none())).unwrap();
    assert_eq!(4, tables.len());
    let_vec!(table1, table2, table3, table4 = tables);
    assert_eq!(id1, table1.id);
    assert_eq!(id2, table2.id);
    assert_eq!(id3, table3.id);
    assert_eq!(id5, table4.id);

    let mut tables = sql!(TableSelectExpr.filter(optional_field.is_some())).unwrap();
    assert_eq!(1, tables.len());
    let_vec!(table1 = tables);
    assert_eq!(id4, table1.id);

    let mut tables = sql!(TableSelectExpr.filter(datetime.year() == 2015)).unwrap();
    assert_eq!(1, tables.len());
    let_vec!(table1 = tables);
    assert_eq!(id4, table1.id);

    let mut tables = sql!(TableSelectExpr.filter(datetime.month() == 11)).unwrap();
    assert_eq!(5, tables.len());
    let_vec!(table1, table2, table3, table4, table5 = tables);
    assert_eq!(id1, table1.id);
    assert_eq!(id2, table2.id);
    assert_eq!(id3, table3.id);
    assert_eq!(id4, table4.id);
    assert_eq!(id5, table5.id);

    // NOTE: the hour is 20 instead of 15 because of the timezone.
    let mut tables = sql!(TableSelectExpr.filter(datetime.year() == 2015 && datetime.month() == 11 &&
        datetime.day() == 16 && datetime.hour() == 20 && datetime.minute() == 51 && datetime.second() > 0)).unwrap();
    assert_eq!(1, tables.len());
    let_vec!(table1 = tables);
    assert_eq!(id4, table1.id);

    let mut tables = sql!(TableSelectExpr.filter(field1.contains("value1"))).unwrap();
    assert_eq!(1, tables.len());
    let_vec!(table1 = tables);
    assert_eq!(id1, table1.id);

    let mut tables = sql!(TableSelectExpr.filter(field1.contains("alue"))).unwrap();
    assert_eq!(5, tables.len());
    let_vec!(table1, table2, table3, table4, table5 = tables);
    assert_eq!(id1, table1.id);
    assert_eq!(id2, table2.id);
    assert_eq!(id3, table3.id);
    assert_eq!(id4, table4.id);
    assert_eq!(id5, table5.id);

    let mut tables = sql!(TableSelectExpr.filter(field1.ends_with("e1"))).unwrap();
    assert_eq!(1, tables.len());
    let_vec!(table1 = tables);
    assert_eq!(id1, table1.id);

    let mut tables = sql!(TableSelectExpr.filter(field1.starts_with("va"))).unwrap();
    assert_eq!(5, tables.len());
    let_vec!(table1, table2, table3, table4, table5 = tables);
    assert_eq!(id1, table1.id);
    assert_eq!(id2, table2.id);
    assert_eq!(id3, table3.id);
    assert_eq!(id4, table4.id);
    assert_eq!(id5, table5.id);

    let tables = sql!(TableSelectExpr.filter(field1.starts_with("e1"))).unwrap();
    assert_eq!(0, tables.len());

    let tables = sql!(TableSelectExpr.filter(field1.ends_with("va"))).unwrap();
    assert_eq!(0, tables.len());

    let value = "alue";
    let mut tables = sql!(TableSelectExpr.filter(field1.contains(value))).unwrap();
    assert_eq!(5, tables.len());
    let_vec!(table1, table2, table3, table4, table5 = tables);
    assert_eq!(id1, table1.id);
    assert_eq!(id2, table2.id);
    assert_eq!(id3, table3.id);
    assert_eq!(id4, table4.id);
    assert_eq!(id5, table5.id);

    let mut tables = sql!(TableSelectExpr.filter(field1.len() == 6)).unwrap();
    assert_eq!(5, tables.len());
    let_vec!(table1, table2, table3, table4, table5 = tables);
    assert_eq!(id1, table1.id);
    assert_eq!(id2, table2.id);
    assert_eq!(id3, table3.id);
    assert_eq!(id4, table4.id);
    assert_eq!(id5, table5.id);

    #[cfg(feature = "postgres")]
    {
        let mut tables = sql!(TableSelectExpr.filter(field1.regex("%3"))).unwrap();
        assert_eq!(1, tables.len());
        let_vec!(table1 = tables);
        assert_eq!(id3, table1.id);

        let tables = sql!(TableSelectExpr.filter(field1.regex("%E3"))).unwrap();
        assert_eq!(0, tables.len());
    }

    let mut tables = sql!(TableSelectExpr.filter(field1.iregex("%E3"))).unwrap();
    assert_eq!(1, tables.len());
    let_vec!(table1 = tables);
    assert_eq!(id3, table1.id);

    let table = sql!(TableSelectExpr.filter(field1 == "value2").get()).unwrap();
    assert_eq!(id2, table.id);

    let mut tables = sql!(TableSelectExpr.filter(datetime.year() == 2013 && field2 < 100).sort(-field1)).unwrap();
    assert_eq!(3, tables.len());
    let_vec!(table1, table2, table3 = tables);
    assert_eq!(id3, table1.id);
    assert_eq!(id2, table2.id);
    assert_eq!(id1, table3.id);

    let mut tables = sql!(TableSelectExpr.filter(field2 < 100 && datetime.year() == 2013).sort(-field1)).unwrap();
    assert_eq!(3, tables.len());
    let_vec!(table1, table2, table3 = tables);
    assert_eq!(id3, table1.id);
    assert_eq!(id2, table2.id);
    assert_eq!(id1, table3.id);

    let mut tables = sql!(TableSelectExpr.filter(field2 >= 42).sort(field2)).unwrap();
    assert_eq!(3, tables.len());
    let_vec!(table1, table2, table3 = tables);
    assert_eq!(id2, table1.id);
    assert_eq!(id1, table2.id);
    assert_eq!(id5, table3.id);

    let mut tables = sql!(TableSelectExpr.filter(field2 > 10).sort(field2)[1..3]).unwrap();
    assert_eq!(2, tables.len());
    let_vec!(table1, table2 = tables);
    assert_eq!(id4, table1.id);
    assert_eq!(id2, table2.id);

    let table = sql!(TableSelectExpr.get(1)).unwrap();
    assert_eq!(1, table.id);
    assert_eq!("value1", table.field1);
    assert_eq!(55, table.field2);

    let table = sql!(TableSelectExpr.get(id2)).unwrap();
    assert_eq!(id2, table.id);
    assert_eq!("value2", table.field1);
    assert_eq!(42, table.field2);

    let table = sql!(TableSelectExpr.get(field2 == 42)).unwrap();
    assert_eq!(id2, table.id);
    assert_eq!("value2", table.field1);
    assert_eq!(42, table.field2);

    let table = sql!(TableSelectExpr.get(field2 == 43));
    assert!(is_not_found(table));

    let table = sql!(TableSelectExpr.get(field1 == "value2" && field2 == 42)).unwrap();
    assert_eq!(id2, table.id);

    let table = sql!(TableSelectExpr.get((field1 == "value2" && field2 == 42))).unwrap();
    assert_eq!(id2, table.id);

    let table = sql!(TableSelectExpr.get(!(field1 == "value2" && field2 == 42))).unwrap();
    assert_eq!(id1, table.id);

    let table = sql!(TableSelectExpr.get(!(field2 < 24))).unwrap();
    assert_eq!(id1, table.id);

    let mut tables = sql!(TableSelectExpr.all().join(related_field)).unwrap();
    assert_eq!(5, tables.len());
    let_vec!(table1, table2, table3, table4, table5 = tables);
    assert_eq!(id1, table1.id);
    assert_eq!(related_field.id, table1.related_field.unwrap().id);
    assert_eq!(id2, table2.id);
    assert_eq!(related_field.id, table2.related_field.unwrap().id);
    assert_eq!(id3, table3.id);
    assert_eq!(related_field2.id, table3.related_field.unwrap().id);
    assert_eq!(id4, table4.id);
    assert_eq!(related_field2.id, table4.related_field.unwrap().id);
    assert_eq!(id5, table5.id);
    assert_eq!(related_field2.id, table5.related_field.unwrap().id);

    let mut tables = sql!(TableSelectExpr.join(related_field)).unwrap();
    assert_eq!(5, tables.len());
    let_vec!(table1, table2, table3, table4, table5 = tables);
    assert_eq!(id1, table1.id);
    assert_eq!(related_field.id, table1.related_field.unwrap().id);
    assert_eq!(id2, table2.id);
    assert_eq!(related_field.id, table2.related_field.unwrap().id);
    assert_eq!(id3, table3.id);
    assert_eq!(related_field2.id, table3.related_field.unwrap().id);
    assert_eq!(id4, table4.id);
    assert_eq!(related_field2.id, table4.related_field.unwrap().id);
    assert_eq!(id5, table5.id);
    assert_eq!(related_field2.id, table5.related_field.unwrap().id);

    let mut tables = sql!(TableSelectExpr.all()[..2]).unwrap();
    assert_eq!(2, tables.len());
    let_vec!(table1, table2 = tables);
    assert_eq!(id1, table1.id);
    assert_eq!(id2, table2.id);

    let mut tables = sql!(TableSelectExpr[..2]).unwrap();
    assert_eq!(2, tables.len());
    let_vec!(table1, table2 = tables);
    assert_eq!(id1, table1.id);
    assert_eq!(id2, table2.id);

    let mut tables = sql!(TableSelectExpr[1..3]).unwrap();
    assert_eq!(2, tables.len());
    let_vec!(table1, table2 = tables);
    assert_eq!(id2, table1.id);
    assert_eq!(id3, table2.id);

    let table = sql!(TableSelectExpr.all()[2]).unwrap();
    assert_eq!(id3, table.id);

    let table = sql!(TableSelectExpr[2]).unwrap();
    assert_eq!(id3, table.id);

    let table = sql!(TableSelectExpr[42]);
    assert!(is_not_found(table));

    let table = sql!(TableSelectExpr[2 - 1]).unwrap();
    assert_eq!(id2, table.id);

    let mut tables = sql!(TableSelectExpr[..2 - 1]).unwrap();
    assert_eq!(1, tables.len());
    let_vec!(table1 = tables);
    assert_eq!(id1, table1.id);

    let mut tables = sql!(TableSelectExpr[2 - 1..]).unwrap();
    assert_eq!(4, tables.len());
    let_vec!(table1, table2, table3, table4 = tables);
    assert_eq!(id2, table1.id);
    assert_eq!(id3, table2.id);
    assert_eq!(id4, table3.id);
    assert_eq!(id5, table4.id);

    let index = 2i64;
    let table = sql!(TableSelectExpr[index]).unwrap();
    assert_eq!(id3, table.id);

    let index = 1i64;
    let end_index = 3i64;
    let mut tables = sql!(TableSelectExpr[index..end_index]).unwrap();
    assert_eq!(2, tables.len());
    let_vec!(table1, table2 = tables);
    assert_eq!(id2, table1.id);
    assert_eq!(id3, table2.id);

    fn result() -> i64 {
        2
    }

    let table = sql!(TableSelectExpr[result()]).unwrap();
    assert_eq!(id3, table.id);

    let index = 2i64;
    let table = sql!(TableSelectExpr[index + 1]).unwrap();
    assert_eq!(id4, table.id);

    let index = -2;
    let table = sql!(TableSelectExpr[i64::from(-index)]).unwrap();
    assert_eq!(id3, table.id);

    let table2_id = sql!(Table2.insert(field1 = 24, field2 = 42)).unwrap();
    let related1 = sql!(Table2.get(table2_id)).unwrap();
    let table2_id = sql!(Table3.insert(field1 = 25, field2 = 43)).unwrap();
    let related2 = sql!(Table3.get(table2_id)).unwrap();
    let table3_id = sql!(Table4.insert(field1 = 26, field2 = 44)).unwrap();
    let related3 = sql!(Table4.get(table3_id)).unwrap();
    let id1 = sql!(Table1.insert(
        field1 = 1,
        field2 = 55,
        related1 = related1,
        related2 = related2,
        related3 = related3,
    )).unwrap();
    let table1 = sql!(Table1.get(id1).join(related1, related2, related3)).unwrap();
    assert_eq!(table1.field1, 1);
    assert_eq!(table1.field2, 55);
    let table_related1 = table1.related1.unwrap();
    let table_related2 = table1.related2.unwrap();
    let table_related3 = table1.related3.unwrap();
    assert_eq!(table_related1.field1, 24);
    assert_eq!(table_related1.field2, 42);
    assert_eq!(table_related2.field1, 25);
    assert_eq!(table_related2.field2, 43);
    assert_eq!(table_related3.field1, 26);
    assert_eq!(table_related3.field2, 44);

    let table5_id = sql!(Table5.insert(field1 = 24, field2 = 42)).unwrap();
    let table6_id = sql!(Table6.insert(field1 = 24, field2 = 42)).unwrap();
    let table5 = Table5 {
        id: table5_id,
        field1: 24,
        field2: 42,
        //tables6: ManyToMany::new(),
    };
    let table6 = Table6 {
        id: table6_id,
        field1: 24,
        field2: 42,
        //tables5: HasMany::new(),
    };
    sql!(Table5_Table6.insert(table5 = table5, table6 = table6)).unwrap();
    //table5.tables6.add(table6);
    //table5.tables6.all();
    sql!(Table5_Table6.all().join(table5));
}
