import {
  AccessorKeyColumnDefBase,
  flexRender,
  getCoreRowModel,
  IdIdentifier,
  useReactTable,
} from "@tanstack/react-table";
import { FC, RefObject, useState, useEffect } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import { Track } from "../../Types/track";
import { File } from "../../Types/file";

export type TableProps = {
  columns: (AccessorKeyColumnDefBase<Track, string | undefined> &
    Partial<IdIdentifier<Track, string | undefined>>)[];
  tracks: Track[] | File[];
  containerRef: RefObject<HTMLDivElement>;
};

const VirtualizedTable: FC<TableProps> = ({
  columns,
  tracks,
  containerRef,
}) => {
  const [data, setData] = useState([...tracks]);

  useEffect(() => {
    setData([...tracks]);
  }, [tracks]);

  const table = useReactTable({
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    data: data as any,
    columns,
    getCoreRowModel: getCoreRowModel(),
    debugTable: true,
  });

  const { rows } = table.getRowModel();
  const rowVirtualizer = useVirtualizer({
    count: rows.length,
    estimateSize: () => 32, //estimate row height for accurate scrollbar dragging
    getScrollElement: () => containerRef.current,
    //measure dynamic row height, except in firefox because it measures table border height incorrectly
    measureElement:
      typeof window !== "undefined" &&
      navigator.userAgent.indexOf("Firefox") === -1
        ? (element) => element?.getBoundingClientRect().height
        : undefined,
    overscan: 5,
  });

  useEffect(() => {
    rowVirtualizer.measure();
    rowVirtualizer.getVirtualItems();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return (
    <div
      style={{
        height: `${rowVirtualizer.getTotalSize()}px`,
        marginBottom: 100,
      }}
    >
      <table style={{ width: "100%" }}>
        <thead>
          {table.getHeaderGroups().map((headerGroup) => (
            <tr
              key={headerGroup.id}
              style={{ height: 48, color: "rgba(0, 0, 0, 0.54)" }}
            >
              {headerGroup.headers.map((header) => (
                <th
                  key={header.id}
                  style={{
                    textAlign:
                      header.column.columnDef.header !== "#"
                        ? "left"
                        : undefined,
                    width: header.getSize(),
                  }}
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
          {rowVirtualizer.getVirtualItems().map((virtualRow, index) => {
            const row = rows[virtualRow.index];
            return (
              <tr
                data-index={virtualRow.index} //needed for dynamic row height measurement
                ref={(node) => rowVirtualizer.measureElement(node)} //measure dynamic row height
                key={row.id}
                style={{
                  height: `${virtualRow.size}px`,
                  transform: `translateY(${
                    virtualRow.start - index * virtualRow.size
                  }px)`,
                }}
              >
                {row.getVisibleCells().map((cell) => (
                  <td key={cell.id}>
                    {flexRender(cell.column.columnDef.cell, cell.getContext())}
                  </td>
                ))}
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
};

export default VirtualizedTable;
