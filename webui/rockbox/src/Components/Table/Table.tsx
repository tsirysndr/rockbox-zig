import {
  AccessorKeyColumnDefBase,
  flexRender,
  getCoreRowModel,
  IdIdentifier,
  useReactTable,
} from "@tanstack/react-table";
import { FC, useState } from "react";
import { Track } from "../../Types/track";

export type TableProps = {
  columns: (AccessorKeyColumnDefBase<Track, string | undefined> &
    Partial<IdIdentifier<Track, string | undefined>>)[];
  tracks: Track[];
};

const Table: FC<TableProps> = ({ columns, tracks }) => {
  const [data, _setData] = useState<Track[]>(() => [...tracks]);

  const table = useReactTable({
    data,
    columns,
    getCoreRowModel: getCoreRowModel(),
  });

  return (
    <table style={{ width: "100%", marginTop: 31 }}>
      <thead>
        {table.getHeaderGroups().map((headerGroup) => (
          <tr
            key={headerGroup.id}
            style={{ height: 36, color: "rgba(0, 0, 0, 0.54)" }}
          >
            {headerGroup.headers.map((header) => (
              <th key={header.id} style={{ textAlign: "left" }}>
                {header.isPlaceholder
                  ? null
                  : flexRender(
                      header.column.columnDef.header,
                      header.getContext()
                    )}
              </th>
            ))}
          </tr>
        ))}
      </thead>
      <tbody>
        {table.getRowModel().rows.map((row) => (
          <tr key={row.id} style={{ height: 48 }}>
            {row.getVisibleCells().map((cell) => (
              <td key={cell.id}>
                {flexRender(cell.column.columnDef.cell, cell.getContext())}
              </td>
            ))}
          </tr>
        ))}
      </tbody>
    </table>
  );
};

export default Table;
