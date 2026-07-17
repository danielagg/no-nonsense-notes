import type { KeyboardEvent, PointerEvent, RefObject } from "react";
import { Check, GripVertical, Plus, Trash2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { Input } from "@/components/ui/input";
import { parseListItem } from "./model";

interface Props {
  items: string[];
  itemIds: string[];
  draggingIndex: number | null;
  itemRowsRef: RefObject<Array<HTMLDivElement | null>>;
  itemInputRefs: RefObject<Array<HTMLInputElement | null>>;
  onAdd: () => void;
  onInsertAfter: (index: number) => void;
  onUpdate: (index: number, value: string) => void;
  onRemove: (index: number) => void;
  onToggle: (index: number) => void;
  onDragStart: (event: PointerEvent<HTMLButtonElement>, index: number) => void;
  onReorderKeyDown: (event: KeyboardEvent<HTMLButtonElement>, index: number) => void;
}

export function ChecklistEditor(props: Props) {
  return (
    <div className="space-y-3">
      {props.items.map((item, index) => (
        <ChecklistRow key={props.itemIds[index]} item={item} index={index} {...props} />
      ))}
      {props.items.length === 0 && (
        <div className="rounded-md border border-dashed border-primary/20 py-10 text-center font-mono text-sm text-muted-foreground">
          <Check className="mx-auto mb-3 size-5" />
          No items yet.
        </div>
      )}
      <Button variant="outline" className="mt-2" onClick={props.onAdd}>
        <Plus /> Add item
      </Button>
    </div>
  );
}

function ChecklistRow({ item, index, ...props }: Props & { item: string; index: number }) {
  const { isChecked, text, hasCheckboxPrefix } = parseListItem(item);

  return (
    <div
      data-checklist-row
      ref={(row) => {
        props.itemRowsRef.current[index] = row;
      }}
      className={`group flex items-center gap-2 rounded-md border border-primary/10 bg-background/45 p-2 transition-[transform,box-shadow,border-color,background-color] duration-150 focus-within:border-primary/35 ${props.draggingIndex === index ? "relative z-10 scale-[1.015] border-primary/45 bg-card shadow-[0_0_24px_color-mix(in_oklch,var(--primary)_9%,transparent)]" : ""}`}
    >
      <button
        type="button"
        className="grid size-8 shrink-0 touch-none cursor-grab select-none place-items-center rounded-sm text-muted-foreground/55 outline-none transition-colors hover:bg-primary/8 hover:text-primary focus-visible:ring-2 focus-visible:ring-ring/30 active:cursor-grabbing"
        onPointerDown={(event) => props.onDragStart(event, index)}
        onKeyDown={(event) => props.onReorderKeyDown(event, index)}
        aria-label={`Reorder item ${index + 1}`}
        aria-keyshortcuts="ArrowUp ArrowDown"
        title="Hold and drag to reorder"
      >
        <GripVertical className="size-4" />
      </button>
      <Checkbox checked={isChecked} onCheckedChange={() => props.onToggle(index)} className="size-5 rounded-sm" />
      <Input
        ref={(input) => {
          props.itemInputRefs.current[index] = input;
        }}
        value={text}
        onChange={(event) => {
          const prefix = isChecked ? "[x] " : hasCheckboxPrefix ? "[ ] " : "";
          props.onUpdate(index, prefix + event.target.value);
        }}
        onKeyDown={(event) => {
          if (event.key !== "Enter" || event.nativeEvent.isComposing) return;
          event.preventDefault();
          props.onInsertAfter(index);
        }}
        placeholder="List item"
        className={`h-9 flex-1 border-0 bg-transparent shadow-none focus-visible:ring-0 dark:bg-transparent ${isChecked ? "text-muted-foreground line-through" : ""}`}
      />
      <Button
        variant="ghost"
        size="icon-sm"
        className="text-muted-foreground opacity-60 hover:bg-destructive/10 hover:text-destructive sm:opacity-0 sm:group-hover:opacity-100"
        onClick={() => props.onRemove(index)}
        aria-label="Remove item"
      >
        <Trash2 />
      </Button>
    </div>
  );
}
