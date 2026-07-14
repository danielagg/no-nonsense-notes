import { useState } from "react";
import { useMutation, useQuery } from "@tanstack/react-query";
import { signup, signin } from "@/lib/api";
import { useAuth } from "@/lib/auth";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import {
  ArrowRight,
  Check,
  Cloud,
  GitFork,
  LockKeyhole,
  Radio,
  Star,
} from "lucide-react";
import { Brand } from "@/components/brand";
import { ThemeToggle } from "@/components/theme-toggle";

const GITHUB_REPO_URL = "https://github.com/danielagg/no-nonsense-notes";
const GITHUB_REPO_API_URL =
  "https://api.github.com/repos/danielagg/no-nonsense-notes";

async function getGithubStarCount() {
  const response = await fetch(GITHUB_REPO_API_URL);

  if (!response.ok) {
    throw new Error("Could not load GitHub stars");
  }

  const repository = (await response.json()) as { stargazers_count: number };
  return repository.stargazers_count;
}

export function AuthPage() {
  const { login } = useAuth();
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");

  const signupMutation = useMutation({
    mutationFn: () => signup(email, password),
    onSuccess: (data) => login(data.token, data.account_id),
  });

  const signinMutation = useMutation({
    mutationFn: () => signin(email, password),
    onSuccess: (data) => login(data.token, data.account_id),
  });

  const isLoading = signupMutation.isPending || signinMutation.isPending;
  const error = signupMutation.error || signinMutation.error;
  const { data: githubStarCount = 0 } = useQuery({
    queryKey: ["github-star-count"],
    queryFn: getGithubStarCount,
    staleTime: 60 * 60 * 1000,
    retry: false,
    placeholderData: 0,
  });

  return (
    <div className="terminal-grid relative min-h-svh overflow-hidden">
      <div className="pointer-events-none absolute inset-0 bg-[radial-gradient(circle_at_18%_25%,color-mix(in_oklch,var(--primary)_9%,transparent),transparent_30%),radial-gradient(circle_at_82%_75%,color-mix(in_oklch,var(--primary)_5%,transparent),transparent_26%)]" />
      <header className="relative z-10 mx-auto flex h-20 w-full max-w-7xl items-center justify-between border-b border-primary/10 px-5 sm:px-8">
        <Brand />
        <div className="flex items-center gap-2">
          <a
            href={GITHUB_REPO_URL}
            target="_blank"
            rel="noreferrer"
            className="inline-flex h-9 items-center gap-2 rounded-md border border-primary/20 bg-primary/[0.04] px-3 font-heading text-[11px] font-semibold uppercase tracking-[0.08em] text-foreground transition-colors hover:border-primary/40 hover:bg-primary/10"
            aria-label={`View the open-source project on GitHub. ${githubStarCount} stars.`}
          >
            <GitFork className="size-3.5" />
            <span className="hidden sm:inline">Open source</span>
            <span className="inline-flex items-center gap-1 text-primary">
              <Star className="size-3 fill-current" />
              {githubStarCount}
            </span>
          </a>
          <ThemeToggle />
        </div>
      </header>

      <main className="relative z-10 mx-auto grid min-h-[calc(100svh-5rem)] w-full max-w-7xl items-center gap-16 px-5 pb-12 sm:px-8 lg:grid-cols-[1fr_460px] lg:pb-20">
        <section className="hidden max-w-2xl lg:block">
          <div className="mb-7 inline-flex items-center gap-2 border border-primary/25 bg-primary/[0.04] px-3 py-1.5 font-heading text-[11px] font-semibold uppercase tracking-[0.12em] text-primary backdrop-blur">
            <Radio className="size-3.5" />
            <p className="pt-1">
              Local first. End-to-end encrypted. Fast by design.
            </p>
          </div>
          <h1 className="font-heading text-5xl font-semibold leading-[1.08] tracking-[-0.055em] xl:text-6xl">
            Just notes.
            <br />
            <span className="text-primary">Fast. Local. Yours.</span>
          </h1>
          <p className="mt-7 max-w-lg border-l border-primary/35 pl-5 text-base leading-7 text-muted-foreground">
            No wikis, workflows, or AI bolted on. Just notes and lists, built
            around local data and kept deliberately small—so they open instantly
            and stay yours.
          </p>
          <div className="mt-10 grid max-w-lg grid-cols-2 gap-4">
            <Feature
              icon={<LockKeyhole />}
              title="Your notes. Your rules."
              description="Local-first data that stays under your control."
            />
            <Feature
              icon={<Cloud />}
              title="Never lose the thread"
              description="Changes sync quietly between your devices."
            />
          </div>
          <div className="mt-10 flex items-center gap-2 font-heading text-xs uppercase tracking-[0.06em] text-muted-foreground">
            <span className="grid size-5 place-items-center border border-primary/25 bg-primary/10 text-primary">
              <Check className="size-3" strokeWidth={3} />
            </span>
            No bloat. No nonsense.
          </div>
        </section>

        <Card className="terminal-glow mx-auto w-full max-w-md gap-0 rounded-lg border border-primary/20 bg-card/92 py-0 ring-0 backdrop-blur">
          <CardHeader className="px-6 pb-6 pt-7 sm:px-8 sm:pt-8">
            <p className="mb-3 font-heading text-[10px] font-semibold uppercase tracking-[0.16em] text-primary">
              Auth // workspace access
            </p>
            <CardTitle className="font-heading text-2xl font-semibold tracking-[-0.04em]">
              Identify yourself
            </CardTitle>
            <CardDescription className="mt-1">
              Sign in or initialize a new account.
            </CardDescription>
          </CardHeader>
          <CardContent className="px-6 pb-7 sm:px-8 sm:pb-8">
            <Tabs defaultValue="signin">
              <TabsList className="grid h-10 w-full grid-cols-2 rounded-md border border-primary/10 bg-muted/65 p-1">
                <TabsTrigger
                  className="rounded-sm font-heading text-xs"
                  value="signin"
                >
                  Sign in
                </TabsTrigger>
                <TabsTrigger
                  className="rounded-sm font-heading text-xs"
                  value="signup"
                >
                  Create account
                </TabsTrigger>
              </TabsList>

              <TabsContent value="signin" className="mt-6">
                <form
                  onSubmit={(e) => {
                    e.preventDefault();
                    signinMutation.mutate();
                  }}
                  className="space-y-5"
                >
                  <Field
                    id="signin-email"
                    label="Email address"
                    value={email}
                    onChange={setEmail}
                    type="email"
                    placeholder="you@company.com"
                  />
                  <Field
                    id="signin-password"
                    label="Password"
                    value={password}
                    onChange={setPassword}
                    type="password"
                    placeholder="Enter your password"
                  />
                  {error && <ErrorMessage message={error.message} />}
                  <Button
                    type="submit"
                    size="lg"
                    className="w-full"
                    disabled={isLoading}
                  >
                    {isLoading ? "Signing in..." : "Sign in"}
                    {!isLoading && <ArrowRight data-icon="inline-end" />}
                  </Button>
                </form>
              </TabsContent>

              <TabsContent value="signup" className="mt-6">
                <form
                  onSubmit={(e) => {
                    e.preventDefault();
                    signupMutation.mutate();
                  }}
                  className="space-y-5"
                >
                  <Field
                    id="signup-email"
                    label="Email address"
                    value={email}
                    onChange={setEmail}
                    type="email"
                    placeholder="you@company.com"
                  />
                  <Field
                    id="signup-password"
                    label="Password"
                    value={password}
                    onChange={setPassword}
                    type="password"
                    placeholder="Choose a secure password"
                  />
                  {error && <ErrorMessage message={error.message} />}
                  <Button
                    type="submit"
                    size="lg"
                    className="w-full"
                    disabled={isLoading}
                  >
                    {isLoading ? "Creating account..." : "Create account"}
                    {!isLoading && <ArrowRight data-icon="inline-end" />}
                  </Button>
                </form>
              </TabsContent>
            </Tabs>
            <p className="mt-6 text-center font-heading text-[10px] uppercase tracking-[0.08em] text-muted-foreground">
              Encrypted transport // local-first data
            </p>
          </CardContent>
        </Card>
      </main>
    </div>
  );
}

function Feature({
  icon,
  title,
  description,
}: {
  icon: React.ReactNode;
  title: string;
  description: string;
}) {
  return (
    <div className="border border-primary/15 bg-card/55 p-4 backdrop-blur">
      <span className="mb-4 grid size-9 place-items-center border border-primary/20 bg-primary/[0.06] text-primary [&_svg]:size-4">
        {icon}
      </span>
      <p className="font-heading text-sm font-semibold uppercase tracking-[0.04em]">
        {title}
      </p>
      <p className="mt-1 text-sm leading-5 text-muted-foreground">
        {description}
      </p>
    </div>
  );
}

function ErrorMessage({ message }: { message: string }) {
  return (
    <p className="rounded-sm border border-destructive/25 bg-destructive/10 px-3 py-2 font-mono text-xs text-destructive">
      {message}
    </p>
  );
}

function Field({
  id,
  label,
  value,
  onChange,
  type,
  placeholder,
}: {
  id: string;
  label: string;
  value: string;
  onChange: (v: string) => void;
  type: string;
  placeholder: string;
}) {
  return (
    <div className="space-y-2">
      <Label
        htmlFor={id}
        className="font-heading text-[11px] font-semibold uppercase tracking-[0.08em] text-muted-foreground"
      >
        {label}
      </Label>
      <Input
        id={id}
        type={type}
        value={value}
        placeholder={placeholder}
        className="h-11 rounded-md bg-background/60 px-3 font-mono"
        onChange={(e: React.ChangeEvent<HTMLInputElement>) =>
          onChange(e.target.value)
        }
        required
      />
    </div>
  );
}
