import type { TreeNode } from "@/types/database";
import { matchSidebarLabel } from "@/lib/sidebarSearch";

const preserveMatchedSubtreeTypes = new Set(["table", "view"]);

const normalizedLabelCache = new WeakMap<TreeNode, { label: string; normalized: string }>();

function normalizedLabel(node: TreeNode): string {
  const cached = normalizedLabelCache.get(node);
  if (cached?.label === node.label) return cached.normalized;

  const normalized = node.label.toLowerCase();
  normalizedLabelCache.set(node, { label: node.label, normalized });
  return normalized;
}

export function filterSidebarTree(nodes: TreeNode[], query: string, collapsedIds: ReadonlySet<string>): TreeNode[] {
  const filteredNodes: { node: TreeNode; score: number }[] = [];

  for (const node of nodes) {
    if (node.type === "object-browser" && node.hiddenChildren) {
      const matches = node.hiddenChildren
        .map((child) => ({ node: child, score: matchSidebarLabel(normalizedLabel(child), query)?.score ?? 0 }))
        .filter((match) => match.score > 0);
      filteredNodes.push(...matches);
      continue;
    }

    const label = normalizedLabel(node);
    const selfMatch = matchSidebarLabel(label, query);
    const preservesSubtree = !!selfMatch && preserveMatchedSubtreeTypes.has(node.type);
    const filteredChildren = preservesSubtree
      ? node.children
      : node.children
        ? filterSidebarTree(node.children, query, collapsedIds)
        : undefined;

    if (selfMatch || (filteredChildren && filteredChildren.length > 0)) {
      if (!node.children) {
        filteredNodes.push({ node, score: selfMatch?.score ?? 0 });
      } else {
        const children = filteredChildren ?? [];
        filteredNodes.push({
          node: {
            ...node,
            children,
            isExpanded: children.length > 0 && !collapsedIds.has(node.id),
          },
          score: selfMatch?.score ?? 0,
        });
      }
    }
  }

  filteredNodes.sort((a, b) => b.score - a.score);
  return filteredNodes.map((match) => match.node);
}
