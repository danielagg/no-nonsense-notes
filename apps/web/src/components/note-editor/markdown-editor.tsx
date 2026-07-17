import ReactMarkdown from "react-markdown";
import { Textarea } from "@/components/ui/textarea";
import type { MarkdownMode } from "./model";

export function MarkdownEditor({
  content,
  mode,
  onChange,
}: {
  content: string;
  mode: MarkdownMode;
  onChange: (content: string) => void;
}) {
  if (mode === "edit") {
    return (
      <Textarea
        value={content}
        onChange={(event) => onChange(event.target.value)}
        placeholder="Start writing..."
        className="min-h-[60vh] resize-none rounded-none border-0 bg-transparent px-0 py-0 font-mono text-[15px] leading-7 shadow-none focus-visible:ring-0 dark:bg-transparent"
      />
    );
  }

  return (
    <div role="tabpanel" aria-label="Markdown preview" className="markdown-preview min-h-[60vh] text-[15px] leading-7">
      <ReactMarkdown>{content || "_Nothing here yet._"}</ReactMarkdown>
    </div>
  );
}
