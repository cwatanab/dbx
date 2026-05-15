import { strict as assert } from "node:assert";
import test from "node:test";
import { filterSidebarTree } from "../src/lib/sidebarSearchTree.ts";
import type { TreeNode } from "../src/types/database.ts";

test("preserves loaded table children when the table itself matches search", () => {
  const nodes: TreeNode[] = [
    {
      id: "conn:db",
      label: "app",
      type: "database",
      connectionId: "conn",
      database: "app",
      isExpanded: true,
      children: [
        {
          id: "conn:db:orders",
          label: "orders",
          type: "table",
          connectionId: "conn",
          database: "app",
          isExpanded: true,
          children: [
            {
              id: "conn:db:orders:__columns",
              label: "tree.columns",
              type: "group-columns",
              connectionId: "conn",
              database: "app",
              tableName: "orders",
              isExpanded: true,
              children: [
                {
                  id: "conn:db:orders:__columns:id",
                  label: "id",
                  type: "column",
                  connectionId: "conn",
                  database: "app",
                  tableName: "orders",
                },
              ],
            },
          ],
        },
      ],
    },
  ];

  const filtered = filterSidebarTree(nodes, "orders", new Set());

  const table = filtered[0]?.children?.[0];
  assert.equal(table?.label, "orders");
  assert.equal(table?.children?.[0]?.label, "tree.columns");
  assert.equal(table?.children?.[0]?.children?.[0]?.label, "id");
});
