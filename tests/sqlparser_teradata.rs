// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

//! Test SQL syntax, specific to [sqlparser::dialect::TeradataDialect].

use sqlparser::dialect::{Dialect, TeradataDialect};
use sqlparser::test_utils::all_dialects_where;
use test_utils::TestedDialects;

mod test_utils;

fn teradata() -> TestedDialects {
    TestedDialects::new(vec![Box::new(TeradataDialect)])
}

#[test]
fn dialect_methods() {
    let d: &dyn Dialect = &TeradataDialect;
    assert_eq!(d.identifier_quote_style("x"), Some('"'));
    assert!(d.is_delimited_identifier_start('"'));
    assert!(!d.is_delimited_identifier_start('`'));
    assert!(d.is_identifier_start('$'));
    assert!(d.is_identifier_start('#'));
    assert!(d.is_identifier_part('$'));
    assert!(d.is_identifier_part('#'));
    assert!(d.supports_group_by_expr());
    assert!(!d.supports_boolean_literals());
    assert!(d.supports_comment_on());
    assert!(d.supports_create_table_select());
    assert!(d.supports_execute_immediate());
    assert!(d.supports_top_before_distinct());
    assert!(d.supports_window_function_null_treatment_arg());
    assert!(d.supports_string_literal_concatenation());
    assert!(d.supports_leading_comma_before_table_options());
}

#[test]
fn parse_identifier() {
    teradata().verified_stmt(concat!(
        "SELECT ",
        "NULL AS foo, ",
        "NULL AS _bar, ",
        "NULL AS #baz, ",
        "NULL AS $qux, ",
        "NULL AS a$1, ",
        "NULL AS a#1, ",
        "NULL AS a_1, ",
        r#"NULL AS "quoted id""#
    ));
}

#[test]
fn parse_create_table_multiset() {
    teradata().verified_stmt("CREATE MULTISET TABLE foo (id INT)");
    teradata().verified_stmt("CREATE SET TABLE foo (id INT)");
}

#[test]
fn parse_create_view_as_of() {
    teradata()
        .verified_stmt("CREATE VIEW v AS AS OF TIMESTAMP '2020-01-01 00:00:00' SELECT * FROM t");
    teradata().verified_stmt("CREATE VIEW v AS AS OF DATE '2020-01-01' SELECT * FROM t");
    teradata().verified_stmt("CREATE VIEW v AS AS OF CURRENT_TIMESTAMP SELECT * FROM t");
    teradata().verified_stmt(
        "REPLACE VIEW v AS LOCKING ROW FOR ACCESS AS OF TIMESTAMP '2020-01-01 00:00:00' SELECT * FROM t",
    );
}

#[test]
fn parse_create_view_locking_clause() {
    teradata().verified_stmt("CREATE VIEW v AS LOCKING TABLE x FOR READ SELECT * FROM x");
    teradata().verified_stmt("CREATE VIEW v AS LOCKING ROW FOR ACCESS NOWAIT SELECT * FROM x");
    teradata().verified_stmt(
        "CREATE VIEW v AS LOCKING TABLE a FOR ACCESS LOCKING TABLE b FOR READ SELECT * FROM a, b",
    );
    teradata().verified_stmt("CREATE VIEW v AS LOCKING ROW IN ACCESS SELECT * FROM x");
    teradata()
        .verified_stmt("CREATE VIEW v AS LOCKING DATABASE db FOR EXCLUSIVE SELECT * FROM db.t");
    teradata().verified_stmt("CREATE VIEW v AS LOCKING FOR SHARE SELECT * FROM x");
    teradata()
        .verified_stmt("CREATE VIEW v AS LOCKING TABLE a FOR ACCESS MODE NOWAIT SELECT * FROM a");
    teradata().verified_stmt("CREATE VIEW v AS LOCKING ROW FOR WRITE MODE SELECT * FROM x");
    teradata().verified_stmt("CREATE VIEW v AS LOCK ROW FOR ACCESS SELECT * FROM x");
    teradata().verified_stmt(
        "CREATE VIEW v AS LOCK TABLE a FOR ACCESS LOCKING ROW FOR READ SELECT * FROM a",
    );
}

#[test]
fn parse_create_table_volatile() {
    teradata().verified_stmt("CREATE VOLATILE TABLE foo (id INT)");
    teradata().verified_stmt("CREATE MULTISET VOLATILE TABLE foo (id INT)");
}

#[test]
fn parse_create_table_fallback() {
    teradata().verified_stmt("CREATE TABLE foo, FALLBACK (id INT)");
    teradata().verified_stmt("CREATE TABLE foo, NO FALLBACK (id INT)");
    teradata().verified_stmt("CREATE MULTISET TABLE foo, NO FALLBACK (id INT)");
}

#[test]
fn parse_create_table_primary_index() {
    teradata().verified_stmt("CREATE TABLE foo (id INT) PRIMARY INDEX (id)");
    teradata().verified_stmt("CREATE TABLE foo (id INT) UNIQUE PRIMARY INDEX (id)");
    teradata().verified_stmt("CREATE TABLE foo (id INT, a INT) PRIMARY INDEX pk (id, a)");
    teradata().verified_stmt("CREATE TABLE foo (id INT) NO PRIMARY INDEX");
}

#[test]
fn parse_create_table_as_with_data() {
    teradata().verified_stmt("CREATE TABLE foo AS (SELECT 1 AS a) WITH DATA");
    teradata().verified_stmt("CREATE TABLE foo AS (SELECT 1 AS a) WITH NO DATA");
    teradata().verified_stmt("CREATE TABLE foo AS (SELECT 1 AS a) WITH DATA AND STATISTICS");
    teradata().verified_stmt("CREATE TABLE foo AS (SELECT 1 AS a) WITH DATA AND NO STATISTICS");
    teradata().verified_stmt("CREATE TABLE foo AS (SELECT 1 AS a) WITH NO DATA AND STATISTICS");
    teradata().verified_stmt("CREATE TABLE foo AS (SELECT 1 AS a) WITH NO DATA AND NO STATISTICS");
}

#[test]
fn parse_create_table_journal() {
    teradata().verified_stmt("CREATE TABLE foo, BEFORE JOURNAL (id INT)");
    teradata().verified_stmt("CREATE TABLE foo, NO BEFORE JOURNAL (id INT)");
    teradata().verified_stmt("CREATE TABLE foo, DUAL BEFORE JOURNAL (id INT)");
    teradata().verified_stmt("CREATE TABLE foo, AFTER JOURNAL (id INT)");
    teradata().verified_stmt("CREATE TABLE foo, NO AFTER JOURNAL (id INT)");
    teradata().verified_stmt("CREATE TABLE foo, DUAL AFTER JOURNAL (id INT)");
    teradata().verified_stmt("CREATE TABLE foo, LOCAL AFTER JOURNAL (id INT)");
    teradata().verified_stmt("CREATE TABLE foo, NOT LOCAL AFTER JOURNAL (id INT)");
    teradata().verified_stmt("CREATE TABLE foo, WITH JOURNAL TABLE = jlog (id INT)");
}

#[test]
fn parse_create_table_checksum() {
    for level in ["DEFAULT", "ON", "OFF", "IMMEDIATE"] {
        teradata().verified_stmt(&format!("CREATE TABLE foo, CHECKSUM = {level} (id INT)"));
    }
}

#[test]
fn parse_create_table_merge_block_ratio() {
    teradata().verified_stmt("CREATE TABLE foo, DEFAULT MERGEBLOCKRATIO (id INT)");
    teradata().verified_stmt("CREATE TABLE foo, NO MERGEBLOCKRATIO (id INT)");
    teradata().verified_stmt("CREATE TABLE foo, MERGEBLOCKRATIO = 60 (id INT)");
    teradata().verified_stmt("CREATE TABLE foo, MERGEBLOCKRATIO = 60 PERCENT (id INT)");
}

#[test]
fn parse_create_table_data_block_size() {
    teradata().verified_stmt("CREATE TABLE foo, MINIMUM DATABLOCKSIZE (id INT)");
    teradata().verified_stmt("CREATE TABLE foo, MAXIMUM DATABLOCKSIZE (id INT)");
    teradata().verified_stmt("CREATE TABLE foo, DEFAULT DATABLOCKSIZE (id INT)");
    teradata().verified_stmt("CREATE TABLE foo, DATABLOCKSIZE = 12582912 (id INT)");
    teradata().verified_stmt("CREATE TABLE foo, DATABLOCKSIZE = 12582912 BYTES (id INT)");
    teradata().verified_stmt("CREATE TABLE foo, DATABLOCKSIZE = 12 KBYTES (id INT)");
    teradata().verified_stmt("CREATE TABLE foo, DATABLOCKSIZE = 12 KILOBYTES (id INT)");
}

#[test]
fn parse_create_table_free_space() {
    teradata().verified_stmt("CREATE TABLE foo, FREESPACE = 0 (id INT)");
    teradata().verified_stmt("CREATE TABLE foo, FREESPACE = 5 PERCENT (id INT)");
}

#[test]
fn parse_create_table_log() {
    teradata().verified_stmt("CREATE TABLE foo, LOG (id INT)");
    teradata().verified_stmt("CREATE VOLATILE TABLE foo, NO LOG (id INT)");
}

#[test]
fn parse_create_table_block_compression() {
    for v in ["DEFAULT", "MANUAL", "NEVER", "ALWAYS", "AUTOTEMP"] {
        teradata().verified_stmt(&format!(
            "CREATE TABLE foo, BLOCKCOMPRESSION = {v} (id INT)"
        ));
    }
    for v in ["ZLIB", "ELZS_H", "DEFAULT"] {
        teradata().verified_stmt(&format!(
            "CREATE TABLE foo, BLOCKCOMPRESSIONALGORITHM = {v} (id INT)"
        ));
    }
    teradata().verified_stmt("CREATE TABLE foo, BLOCKCOMPRESSIONLEVEL = DEFAULT (id INT)");
    teradata().verified_stmt("CREATE TABLE foo, BLOCKCOMPRESSIONLEVEL = 5 (id INT)");
}

#[test]
fn parse_create_table_map() {
    teradata().verified_stmt("CREATE TABLE foo, MAP = amp_map (id INT)");
    teradata().verified_stmt("CREATE TABLE foo, MAP = amp_map COLOCATE USING other_tbl (id INT)");
}

#[test]
fn parse_create_table_isolated_loading() {
    teradata().verified_stmt("CREATE TABLE foo, WITH ISOLATED LOADING (id INT)");
    teradata().verified_stmt("CREATE TABLE foo, WITH NO ISOLATED LOADING (id INT)");
    teradata().verified_stmt("CREATE TABLE foo, WITH CONCURRENT ISOLATED LOADING (id INT)");
    teradata().verified_stmt("CREATE TABLE foo, WITH NO CONCURRENT ISOLATED LOADING (id INT)");
    for f in ["ALL", "INSERT", "NONE"] {
        teradata().verified_stmt(&format!(
            "CREATE TABLE foo, WITH CONCURRENT ISOLATED LOADING FOR {f} (id INT)"
        ));
        teradata().verified_stmt(&format!(
            "CREATE TABLE foo, WITH ISOLATED LOADING FOR {f} (id INT)"
        ));
    }
}

#[test]
fn parse_create_table_partition_by() {
    teradata().verified_stmt("CREATE TABLE foo (id INT) PARTITION BY COLUMN");
    teradata().verified_stmt(
        "CREATE TABLE foo (id INT) PARTITION BY RANGE_N(id BETWEEN 1 AND 100 EACH 10)",
    );
    teradata().verified_stmt(
        "CREATE TABLE foo (id INT, col VARCHAR(10)) PARTITION BY CASE_N(col = 'A', col = 'B', NO CASE, UNKNOWN)",
    );
}

#[test]
fn parse_range_n() {
    let d = all_dialects_where(|d| d.supports_range_function());
    d.verified_expr("RANGE_N(id BETWEEN 1 AND 100)");
    d.verified_expr("RANGE_N(id BETWEEN 1 AND 100 EACH 10)");
    d.verified_expr("RANGE_N(id BETWEEN * AND 100)");
    d.verified_expr("RANGE_N(id BETWEEN 1 AND *)");
    d.verified_expr("RANGE_N(id BETWEEN * AND *)");
    d.verified_expr("RANGE_N(id BETWEEN * AND 100 EACH 10)");
    d.verified_expr("RANGE_N(id BETWEEN 1 AND 100, UNKNOWN)");
    d.verified_expr("RANGE_N(id BETWEEN 1 AND 100, NO RANGE)");
    d.verified_expr("RANGE_N(id BETWEEN 1 AND 100, NO RANGE OR UNKNOWN)");
    d.verified_expr("RANGE_N(id BETWEEN 1 AND 100 EACH 10, NO RANGE, UNKNOWN)");
    d.verified_expr("RANGE_N(id BETWEEN 1 AND 100 EACH 10, 200 AND 300 EACH 20)");
    d.verified_expr("RANGE_N(id BETWEEN 1 AND 100, 200 AND 300)");
}

#[test]
fn parse_case_n() {
    let d = all_dialects_where(|d| d.supports_range_function());
    d.verified_expr("CASE_N(col = 'A', col = 'B')");
    d.verified_expr("CASE_N(col = 'A', col = 'B', UNKNOWN)");
    d.verified_expr("CASE_N(col = 'A', col = 'B', NO CASE)");
    d.verified_expr("CASE_N(col = 'A', col = 'B', NO CASE OR UNKNOWN)");
    d.verified_expr("CASE_N(col = 'A', col = 'B', NO CASE, UNKNOWN)");
}

#[test]
fn parse_create_table_column_options() {
    // COMPRESS family
    teradata().verified_stmt("CREATE TABLE t (a INT COMPRESS)");
    teradata().verified_stmt("CREATE TABLE t (a INT NO COMPRESS)");
    teradata().verified_stmt("CREATE TABLE t (a INT AUTO COMPRESS)");
    teradata().verified_stmt("CREATE TABLE t (a INT COMPRESS (1, 2, 3))");
    teradata().verified_stmt("CREATE TABLE t (a INT COMPRESS USING zlib)");
    teradata().verified_stmt("CREATE TABLE t (a INT COMPRESS USING zlib DECOMPRESS USING unzlib)");
    // CASESPECIFIC
    teradata().verified_stmt("CREATE TABLE t (a VARCHAR(10) CASESPECIFIC)");
    teradata().verified_stmt("CREATE TABLE t (a VARCHAR(10) NOT CASESPECIFIC)");
    // UPPERCASE
    teradata().verified_stmt("CREATE TABLE t (a VARCHAR(10) UPPERCASE)");
    teradata().verified_stmt("CREATE TABLE t (a VARCHAR(10) NOT UPPERCASE)");
    // FORMAT
    teradata().verified_stmt("CREATE TABLE t (a DATE FORMAT 'YYYY-MM-DD')");
    // TITLE
    teradata().verified_stmt("CREATE TABLE t (a INT TITLE 'The Column')");
    // NAMED
    teradata().verified_stmt("CREATE TABLE t (a INT NAMED other_name)");
    // WITH DEFAULT
    teradata().verified_stmt("CREATE TABLE t (a INT WITH DEFAULT)");
    // Combined
    teradata().verified_stmt(concat!(
        "CREATE TABLE t (",
        "a INT NOT NULL COMPRESS (1, 2, 3), ",
        "b VARCHAR(10) CHARACTER SET LATIN CASESPECIFIC, ",
        "c DATE FORMAT 'YYYY-MM-DD' TITLE 'Birth Date', ",
        "d INT AUTO COMPRESS NAMED other_d",
        ")"
    ));
}

#[test]
fn parse_create_table_as_source_table() {
    teradata().verified_stmt("CREATE TABLE t AS source_tbl");
    teradata().verified_stmt("CREATE TABLE t AS schema.source_tbl");
    teradata().verified_stmt("CREATE TABLE t AS source_tbl WITH DATA");
    teradata().verified_stmt("CREATE TABLE t AS source_tbl WITH NO DATA");
    teradata().verified_stmt("CREATE TABLE t AS source_tbl WITH DATA AND STATISTICS");
}

#[test]
fn parse_create_table_secondary_indexes() {
    teradata().verified_stmt("CREATE TABLE t (a INT, b INT) INDEX (b)");
    teradata().verified_stmt("CREATE TABLE t (a INT, b INT) UNIQUE INDEX (b)");
    teradata().verified_stmt("CREATE TABLE t (a INT, b INT) INDEX idx_b (b)");
    teradata().verified_stmt("CREATE TABLE t (a INT, b INT) UNIQUE INDEX idx_b (b)");
    teradata().verified_stmt("CREATE TABLE t (a INT, b INT, c INT) INDEX (b) UNIQUE INDEX (c)");
    // Multiple indexes
    teradata().verified_stmt("CREATE TABLE t (a INT, b INT, c INT) INDEX (b) UNIQUE INDEX (c)");
    teradata().verified_stmt("CREATE TABLE t (a INT, b INT) INDEX (b) ORDER BY VALUES(b)");
    teradata().verified_stmt("CREATE TABLE t (a INT, b INT) INDEX (b) ORDER BY HASH(b)");
    teradata().verified_stmt("CREATE TABLE t (a INT, b INT) INDEX (b) ORDER BY (b)");
    teradata().verified_stmt("CREATE TABLE t (a INT, b INT) UNIQUE PRIMARY INDEX (a) INDEX (b)");
    teradata().verified_stmt("CREATE TABLE t AS source_tbl INDEX (b)");
}

#[test]
fn parse_create_table_foreign_key_with_check_option() {
    teradata().verified_stmt(
        "CREATE TABLE t (a INT, FOREIGN KEY (a) REFERENCES other(id) WITH CHECK OPTION)",
    );
    teradata().verified_stmt(
        "CREATE TABLE t (a INT, FOREIGN KEY (a) REFERENCES other(id) WITH NO CHECK OPTION)",
    );
    // Column-level REFERENCES with WITH CHECK OPTION
    teradata().verified_stmt("CREATE TABLE t (a INT REFERENCES other (id) WITH NO CHECK OPTION)");
}

#[test]
fn parse_create_table_combined() {
    teradata().verified_stmt(concat!(
        "CREATE MULTISET VOLATILE TABLE foo, NO FALLBACK, NO BEFORE JOURNAL, ",
        "NO AFTER JOURNAL, CHECKSUM = DEFAULT, DEFAULT MERGEBLOCKRATIO, ",
        "DATABLOCKSIZE = 12582912 BYTES, FREESPACE = 0 PERCENT, ",
        "BLOCKCOMPRESSION = AUTOTEMP, MAP = amp_map COLOCATE USING other_tbl, ",
        "WITH CONCURRENT ISOLATED LOADING FOR ALL ",
        "(id INT, name VARCHAR(100)) ",
        "UNIQUE PRIMARY INDEX (id) ",
        "ON COMMIT PRESERVE ROWS"
    ));
}

#[test]
fn parse_leading_comma_before_table_options() {
    let dialect = all_dialects_where(|d| d.supports_leading_comma_before_table_options());
    dialect.verified_stmt("CREATE TABLE foo, FALLBACK (id INT)");

    let unsupported_dialects =
        all_dialects_where(|d| !d.supports_leading_comma_before_table_options());
    assert!(unsupported_dialects
        .parse_sql_statements("CREATE TABLE foo, FALLBACK (id INT)")
        .is_err());
}

#[test]
fn parse_create_view() {
    teradata().verified_stmt("CREATE VIEW foo AS SELECT a FROM bar");
    teradata().verified_stmt("CREATE VIEW foo (a, b) AS SELECT a, b FROM bar");
}

#[test]
fn parse_replace_view() {
    teradata().verified_stmt("REPLACE VIEW foo AS SELECT a FROM bar");
    teradata().verified_stmt("REPLACE VIEW foo (a, b) AS SELECT a, b FROM bar");
}

#[test]
fn parse_create_recursive_view() {
    teradata().verified_stmt("CREATE RECURSIVE VIEW foo AS SELECT a FROM bar");
    teradata().verified_stmt("REPLACE RECURSIVE VIEW foo AS SELECT a FROM bar");
}

#[test]
fn parse_typed_view_columns() {
    let dialects = all_dialects_where(|d| d.supports_typed_view_columns());
    dialects.verified_stmt(
        "CREATE RECURSIVE VIEW foo (id INTEGER, n INTEGER) AS SELECT id, n FROM bar",
    );
    dialects.verified_stmt("CREATE VIEW foo (id INTEGER) AS SELECT id FROM bar");
}

#[test]
fn parse_create_view_with_check_option() {
    teradata().verified_stmt("CREATE VIEW foo AS SELECT a FROM bar WITH CHECK OPTION");
    teradata().verified_stmt("CREATE VIEW foo AS SELECT a FROM bar WITH CASCADED CHECK OPTION");
    teradata().verified_stmt("CREATE VIEW foo AS SELECT a FROM bar WITH LOCAL CHECK OPTION");
    teradata().verified_stmt("REPLACE VIEW foo AS SELECT a FROM bar WITH CHECK OPTION");
    teradata().verified_stmt(
        "REPLACE RECURSIVE VIEW foo AS SELECT a FROM bar WITH CASCADED CHECK OPTION",
    );
}

#[test]
fn parse_create_join_index() {
    teradata().verified_stmt("CREATE JOIN INDEX ji AS SELECT a, b FROM t");
    teradata().verified_stmt("CREATE JOIN INDEX ji AS SELECT a, b FROM t PRIMARY INDEX (a)");
    teradata().verified_stmt("CREATE JOIN INDEX ji AS SELECT a FROM t NO PRIMARY INDEX");
    teradata().verified_stmt("CREATE JOIN INDEX ji AS SELECT a FROM t UNIQUE PRIMARY INDEX (a)");
    teradata().verified_stmt("CREATE JOIN INDEX ji AS SELECT a FROM t PARTITION BY a");
    teradata().verified_stmt("CREATE JOIN INDEX ji AS SELECT a FROM t INDEX (a) INDEX (b)");
    teradata().verified_stmt(concat!(
        "CREATE JOIN INDEX ji, FALLBACK, CHECKSUM = ON ",
        "AS SELECT a FROM t PRIMARY INDEX (a)"
    ));
}

#[test]
fn parse_interval_qualifier_suffix() {
    teradata().verified_stmt("SELECT a YEAR TO MONTH");
    teradata().verified_stmt(concat!(
        "SELECT 1 WHERE ",
        "((CURRENT_TIMESTAMP - CURRENT_TIMESTAMP) DAY (4) TO MINUTE) > ",
        "INTERVAL '30' MINUTE"
    ));
    teradata().verified_stmt("SELECT (a - b) HOUR");
    teradata().verified_stmt("SELECT (a - b) SECOND (3, 2)");
    teradata().verified_stmt("SELECT (a - b) YEAR TO MONTH");
    teradata().verified_stmt("SELECT a YEAR TO MONTH");
    teradata().verified_stmt("SELECT a DAY (4) TO MINUTE");
}
