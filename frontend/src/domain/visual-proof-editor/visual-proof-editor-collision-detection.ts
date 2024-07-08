import { CollisionDescriptor, CollisionDetection, UniqueIdentifier } from '@dnd-kit/core';
import { VisualProofEditorProofTreeViewId } from './components/visual-proof-editor-proof-tree-view';
import { ClientRect } from '@dnd-kit/core';

type Collision = CollisionDescriptor & { id: UniqueIdentifier };

export const visualProofEditorCollisionDetection: CollisionDetection = ({
    collisionRect,
    droppableRects,
    droppableContainers,
}) => {
    const collisions: Collision[] = [];

    for (const droppableContainer of droppableContainers) {
        const { id } = droppableContainer;
        const rect = droppableRects.get(id);

        if (rect) {
            const intersectionRatio = getIntersectionRatio(rect, collisionRect);

            if (intersectionRatio > 0) {
                collisions.push({
                    id,
                    data: { droppableContainer, value: intersectionRatio },
                });
            }
        }
    }

    return collisions.sort(sortCollisionsDescAndViewAreaLast);
};

function sortCollisionsDescAndViewAreaLast(
    { id: aId, data: { value: a } }: Collision,
    { id: bId, data: { value: b } }: Collision,
) {

    if (aId === VisualProofEditorProofTreeViewId) {
        return 1;
    }

    if (bId === VisualProofEditorProofTreeViewId) {
        return -1;
    }

    return b - a;
}

function getIntersectionRatio(
  entry: ClientRect,
  target: ClientRect
): number {
  const top = Math.max(target.top, entry.top);
  const left = Math.max(target.left, entry.left);
  const right = Math.min(target.left + target.width, entry.left + entry.width);
  const bottom = Math.min(target.top + target.height, entry.top + entry.height);
  const width = right - left;
  const height = bottom - top;

  if (left < right && top < bottom) {
    const targetArea = target.width * target.height;
    const entryArea = entry.width * entry.height;
    const intersectionArea = width * height;
    const intersectionRatio =
      intersectionArea / (targetArea + entryArea - intersectionArea);

    return Number(intersectionRatio.toFixed(4));
  }

  // Rectangles do not overlap, or overlap has an area of zero (edge/corner overlap)
  return 0;
}