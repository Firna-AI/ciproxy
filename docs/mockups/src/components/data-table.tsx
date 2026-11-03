// A data table with the props of the Firna Table. Outside a design direction
// it is the Firna Table. Inside one, it is a dense semantic table: one line
// per row at the row height of the direction, a header band, and hairlines.
// It reads columns, rows, rowKey, cell, headless, loading, loadingRowCount,
// and accessibilityLabel; the Firna density and style props stay Firna's.

import { Table, type TableColumn, type TableProps } from "@firna/ui/table";

import { useDirection } from "../theme/direction.ts";
import "./data-table.css";

function columnClass(column: TableColumn): string {
  const align = column.align && column.align !== "left" ? column.align : "";
  return `col-${column.key}${align ? ` dt-${align}` : ""}`;
}

/**
 * Fixed columns keep their width. Flex columns share the rest by flex, as a
 * preferred width: the automatic table layout still widens a column to its
 * content, so a narrow window never clips a value.
 */
function columnWidth(column: TableColumn, columns: TableColumn[]): string {
  if (column.width !== undefined) return `${column.width}px`;
  const flexTotal = columns
    .filter((c) => c.width === undefined)
    .reduce((sum, c) => sum + (c.flex ?? 1), 0);
  return `${(((column.flex ?? 1) / flexTotal) * 100).toFixed(2)}%`;
}

export function DataTable<Row>(props: TableProps<Row>) {
  const direction = useDirection();
  if (!direction) return <Table {...props} />;
  return <DenseTable {...props} />;
}

function DenseTable<Row>({
  accessibilityLabel,
  cell,
  columns,
  headless,
  loading,
  loadingRowCount = 6,
  rowKey,
  rows,
}: TableProps<Row>) {
  return (
    <table
      className="dt"
      aria-label={accessibilityLabel}
      aria-busy={loading ? true : undefined}
    >
      <colgroup>
        {columns.map((column) => (
          <col
            key={column.key}
            className={`col-${column.key}`}
            style={{ width: columnWidth(column, columns) }}
          />
        ))}
      </colgroup>
      {headless ? null : (
        <thead>
          <tr>
            {columns.map((column) => (
              <th key={column.key} scope="col" className={columnClass(column)}>
                {column.label}
              </th>
            ))}
          </tr>
        </thead>
      )}
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
