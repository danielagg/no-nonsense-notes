import type { ReactNode } from "react";
import { Check, Cloud, LockKeyhole, Radio } from "lucide-react";

export function MarketingPanel() {
  return (
    <section className="hidden max-w-2xl lg:block">
      <div className="mb-7 inline-flex items-center gap-2 border border-primary/25 bg-primary/[0.04] px-3 py-1.5 font-heading text-[11px] font-semibold uppercase tracking-[0.12em] text-primary backdrop-blur">
        <Radio className="size-3.5" /><p className="pt-1">Local first. End-to-end encrypted. Fast by design.</p>
      </div>
      <h1 className="font-heading text-5xl font-semibold leading-[1.08] tracking-[-0.055em] xl:text-6xl">
        Just notes.<br /><span className="text-primary">Fast. Local. Yours.</span>
      </h1>
      <p className="mt-7 max-w-lg border-l border-primary/35 pl-5 text-base leading-7 text-muted-foreground">
        No wikis, workflows, or AI bolted on. Just notes and lists, built around local data and kept deliberately small—so they open instantly and stay yours.
      </p>
      <div className="mt-10 grid max-w-lg grid-cols-2 gap-4">
        <Feature icon={<LockKeyhole />} title="Your notes. Your rules." description="Local-first data that stays under your control." />
        <Feature icon={<Cloud />} title="Never lose the thread" description="Changes sync quietly between your devices." />
      </div>
      <div className="mt-10 flex items-center gap-2 font-heading text-xs uppercase tracking-[0.06em] text-muted-foreground">
        <span className="grid size-5 place-items-center border border-primary/25 bg-primary/10 text-primary"><Check className="size-3" strokeWidth={3} /></span>
        No bloat. No nonsense.
      </div>
    </section>
  );
}

function Feature({ icon, title, description }: { icon: ReactNode; title: string; description: string }) {
  return (
    <div className="border border-primary/15 bg-card/55 p-4 backdrop-blur">
      <span className="mb-4 grid size-9 place-items-center border border-primary/20 bg-primary/[0.06] text-primary [&_svg]:size-4">{icon}</span>
      <p className="font-heading text-sm font-semibold uppercase tracking-[0.04em]">{title}</p>
      <p className="mt-1 text-sm leading-5 text-muted-foreground">{description}</p>
    </div>
  );
}
