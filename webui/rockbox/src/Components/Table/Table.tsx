import {
  AccessorKeyColumnDefBase,
  flexRender,
  getCoreRowModel,
  IdIdentifier,
  useReactTable,
} from "@tanstack/react-table";
import { FC, useEffect, useState } from "react";
import { Track } from "../../Types/track";

export type TableProps = {
  columns: (AccessorKeyColumnDefBase<Track, string | undefined> &
    Partial<IdIdentifier<Track, string | undefined>>)[];
  tracks: Track[];
};

const Table: FC<TableProps> = ({ columns, tracks }) => {
  const [data, setData] = useState<Track[]>(() => [...tracks]);

  useEffect(() => {
    setData([...tracks]);
  }, [tracks]);

  const table = useReactTable({
    data,
    columns,
    getCoreRowModel: getCoreRowModel(),
  });

  return (
    <table style={{ width: "100%", marginTop: 20 }}>
      <thead>
        {table.getHeaderGroups().map((headerGroup) => (
          <tr
            key={headerGroup.id}
            style={{ height: 36, color: "rgba(0, 0, 0, 0.54)" }}
          >
            {headerGroup.headers.map((header) => (
              <th
                key={header.id}
                style={{ textAlign: "left", width: header.getSize() }}
              >
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
              <td
                key={cell.id}
                style={{
                  width: cell.column.getSize(),
                  overflow: "hidden",
                }}
              >
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
