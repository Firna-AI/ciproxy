// A dense data table: one line per row, a header band, and hairlines. Fixed
// columns keep their width; flex columns share the rest as a preferred width,
// and the automatic table layout still widens a column to its content, so no
// value is cut off. While a list loads, a table keeps its header and shows
// bars; a mobile list has no header, so LoadingRows shows bar rows alone.

import type { ReactNode } from "react";

import "./data-table.css";

export type Column = {
  key: string;
  label: string;
  /** A fixed width in px. */
  width?: number;
  /** A share of the width that the fixed columns leave. Defaults to 1. */
  flex?: number;
  align?: "left" | "right";
};

function columnClass(column: Column): string {
  return `col-${column.key}${column.align === "right" ? " dt-right" : ""}`;
}

function columnWidth(column: Column, columns: readonly Column[]): string {
  if (column.width !== undefined) return `${column.width}px`;
  const flexTotal = columns
    .filter((c) => c.width === undefined)
    .reduce((sum, c) => sum + (c.flex ?? 1), 0);
  return `${(((column.flex ?? 1) / flexTotal) * 100).toFixed(2)}%`;
}

export function DataTable<Row>({
  columns,
  rows,
  rowKey,
  cell,
  loading = false,
  loadingRowCount = 6,
}: {
  columns: readonly Column[];
  rows: readonly Row[];
  rowKey: (row: Row, index: number) => string;
  cell: (row: Row, key: string) => ReactNode;
  loading?: boolean;
  loadingRowCount?: number;
}) {
  return (
    <table className="dt" aria-busy={loading ? true : undefined}>
      <colgroup>
        {columns.map((column) => (
          <col
            key={column.key}
            className={`col-${column.key}`}
            style={{ width: columnWidth(column, columns) }}
          />
        ))}
      </colgroup>
      <thead>
        <tr>
          {columns.map((column) => (
            <th key={column.key} scope="col" className={columnClass(column)}>
              {column.label}
            </th>
          ))}
        </tr>
      </thead>
      <tbody>
        {loading
          ? Array.from({ length: loadingRowCount }, (_, i) => (
              <tr key={i} className="dt-loading" aria-hidden="true">
                {columns.map((column) => (
                  <td key={column.key} className={columnClass(column)}>
                    <span className="dt-bar" />
                  </td>
                ))}
              </tr>
            ))
          : rows.map((row, i) => (
              <tr key={rowKey(row, i)}>
                {columns.map((column) => (
                  <td key={column.key} className={columnClass(column)}>
                    {cell(row, column.key)}
                  </td>
                ))}
              </tr>
            ))}
      </tbody>
    </table>
  );
}

/** Bar rows without a header, for a mobile list that is still loading. */
export function LoadingRows({ count }: { count: number }) {
  return (
    <ul className="dt-rows" aria-busy="true">
      {Array.from({ length: count }, (_, i) => (
        <li key={i} className="dt-rows-item" aria-hidden="true">
          <span
            className="dt-bar"
            style={{ width: `${48 + ((i * 17) % 30)}%` }}
          />
          <span className="dt-bar dt-bar-short" />
        </li>
      ))}
    </ul>
  );
}
