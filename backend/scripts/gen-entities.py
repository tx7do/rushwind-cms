#!/usr/bin/env python3
# gen-entities.py — 从黄金 DDL（sql/schema.sql）生成 sea-orm entity 模块树
# （data/entities.rs，一表一 pub mod）。生成物不入手改：schema 变更 → 重新生成。
#
# 类型映射：
#   bigint → i64；integer → i32；smallint → i16；double precision → f64
#   character varying / text → String
#   boolean → bool
#   timestamp with time zone → DateTimeWithTimeZone（with-chrono）
#   jsonb → Json（with-json）
#   bytea → Vec<u8>
import re
import sys

RUST_KEYWORDS = {
    "type", "ref", "fn", "in", "for", "if", "else", "match", "box", "use",
    "as", "break", "const", "continue", "crate", "dyn", "enum", "extern",
    "false", "impl", "let", "loop", "mod", "move", "mut", "pub", "return",
    "self", "static", "struct", "super", "trait", "true", "unsafe", "where",
    "while", "async", "await",
}

TYPE_MAP = [
    (re.compile(r"^bigint$"), "i64"),
    (re.compile(r"^integer$"), "i32"),
    (re.compile(r"^smallint$"), "i16"),
    (re.compile(r"^double precision$"), "f64"),
    (re.compile(r"^character varying(\(\d+\))?$"), "String"),
    (re.compile(r"^text$"), "String"),
    (re.compile(r"^boolean$"), "bool"),
    (re.compile(r"^timestamp with time zone$"), "DateTimeWithTimeZone"),
    (re.compile(r"^jsonb$"), "Json"),
    (re.compile(r"^bytea$"), "Vec<u8>"),
]


def rust_field(name: str) -> str:
    return f"r#{name}" if name in RUST_KEYWORDS else name


def map_type(coltype: str) -> str:
    for pattern, rust in TYPE_MAP:
        if pattern.match(coltype):
            return rust
    return None


def main(schema_path: str, out_path: str) -> None:
    with open(schema_path, encoding="utf-8") as f:
        sql = f.read()

    tables = {}  # name -> [(col, type, notnull)]
    for m in re.finditer(
        r'CREATE TABLE IF NOT EXISTS "([a-z_]+)" \(\n(.*?)\n\);', sql, re.S
    ):
        name, body = m.group(1), m.group(2)
        cols = []
        for line in body.split("\n"):
            cm = re.match(r'^  "([a-z_0-9]+)" ([a-z ]+(?:\(\d+(?:,\d+)?\))?)(.*)$', line)
            if not cm:
                continue
            col, coltype, rest = cm.group(1), cm.group(2).strip(), cm.group(3)
            coltype = re.sub(r"\s+", " ", coltype)
            cols.append((col, coltype, "NOT NULL" in rest))
        tables[name] = cols

    out = []
    out.append("//! Generated sea-orm entity modules — one per table of the")
    out.append("//! golden DDL (sql/schema.sql). DO NOT EDIT; regenerate via")
    out.append("//! scripts/gen-entities.py when the schema is re-dumped.")
    out.append("")
    out.append("#![allow(clippy::all)]")
    out.append("#![allow(missing_docs)]")
    out.append("")

    pk = {}
    for m in re.finditer(
        r'ALTER TABLE "([a-z_]+)" ADD CONSTRAINT "[a-z_]+" PRIMARY KEY \(([a-z_0-9]+)\)',
        sql,
    ):
        pk[m.group(1)] = m.group(2)

    for name in sorted(tables):
        cols = tables[name]
        pk_col = pk.get(name, "id")
        out.append(f"pub mod {name} {{")
        out.append("    use sea_orm::entity::prelude::*;")
        out.append("")
        out.append("    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]")
        out.append(f'    #[sea_orm(table_name = "{name}")]')
        out.append("    pub struct Model {")
        for col, coltype, notnull in cols:
            rust = map_type(coltype)
            if rust is None:
                raise SystemExit(f"unmapped type {coltype!r} on {name}.{col}")
            if col == pk_col:
                if rust != "i64":
                    raise SystemExit(f"unexpected pk type {rust} on {name}.{col}")
                out.append("        #[sea_orm(primary_key)]")
                out.append(f"        pub {rust_field(col)}: i64,")
                continue
            if rust == "String":
                field = f"String" if notnull else "Option<String>"
            elif notnull:
                field = rust
            else:
                field = f"Option<{rust}>"
            out.append(f"        pub {rust_field(col)}: {field},")
        out.append("    }")
        out.append("")
        out.append("    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]")
        out.append("    pub enum Relation {}")
        out.append("")
        out.append("    impl ActiveModelBehavior for ActiveModel {}")
        out.append("}")
        out.append("")

    with open(out_path, "w", encoding="utf-8") as f:
        f.write("\n".join(out))
    print(f"generated {len(tables)} entity modules -> {out_path}")


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
