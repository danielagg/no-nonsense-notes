import { useState } from "react";
import { useMutation } from "@tanstack/react-query";
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
import { ArrowRight, Check, Cloud, LockKeyhole, Sparkles } from "lucide-react";
import { Brand } from "@/components/brand";
import { ThemeToggle } from "@/components/theme-toggle";

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

  return (
    <div className="relative min-h-svh overflow-hidden bg-background">
      <div className="pointer-events-none absolute inset-0 bg-[radial-gradient(circle_at_15%_20%,color-mix(in_oklch,var(--primary)_10%,transparent),transparent_28%),radial-gradient(circle_at_85%_80%,color-mix(in_oklch,var(--primary)_7%,transparent),transparent_25%)]" />
      <header className="relative z-10 mx-auto flex h-20 w-full max-w-7xl items-center justify-between px-5 sm:px-8">
        <Brand />
        <ThemeToggle />
      </header>

      <main className="relative z-10 mx-auto grid min-h-[calc(100svh-5rem)] w-full max-w-7xl items-center gap-16 px-5 pb-12 sm:px-8 lg:grid-cols-[1fr_460px] lg:pb-20">
        <section className="hidden max-w-xl lg:block">
          <div className="mb-6 inline-flex items-center gap-2 rounded-full border bg-card/70 px-3 py-1.5 text-xs font-medium text-muted-foreground shadow-sm backdrop-blur">
            <Sparkles className="size-3.5 text-primary" />A calmer place for
            your best thinking
          </div>
          <h1 className="font-heading text-5xl font-semibold leading-[1.08] tracking-[-0.045em] xl:text-6xl">
            Notes that stay out of your way.
          </h1>
          <p className="mt-6 max-w-lg text-lg leading-8 text-muted-foreground">
            Capture ideas, shape lists, and keep everything in sync—without the
            clutter.
          </p>
          <div className="mt-10 grid max-w-lg grid-cols-2 gap-4">
            <Feature
              icon={<Cloud />}
              title="Always in sync"
              description="Your notes follow you everywhere."
            />
            <Feature
              icon={<LockKeyhole />}
              title="Private by default"
              description="A workspace that belongs to you."
            />
          </div>
          <div className="mt-10 flex items-center gap-2 text-sm text-muted-foreground">
            <span className="grid size-5 place-items-center rounded-full bg-emerald-500/12 text-emerald-600 dark:text-emerald-400">
              <Check className="size-3" strokeWidth={3} />
            </span>
            Simple markdown and lists. Nothing you don&apos;t need.
          </div>
        </section>

        <Card className="mx-auto w-full max-w-md gap-0 rounded-2xl border bg-card/90 py-0 shadow-2xl shadow-foreground/[0.06] ring-0 backdrop-blur dark:shadow-black/30">
          <CardHeader className="px-6 pb-6 pt-7 sm:px-8 sm:pt-8">
            <CardTitle className="font-heading text-2xl font-semibold tracking-[-0.035em]">
              Welcome back
            </CardTitle>
            <CardDescription className="mt-1">
              Sign in or create an account to continue.
            </CardDescription>
          </CardHeader>
          <CardContent className="px-6 pb-7 sm:px-8 sm:pb-8">
            <Tabs defaultValue="signin">
              <TabsList className="grid h-10 w-full grid-cols-2 rounded-xl bg-muted/80 p-1">
                <TabsTrigger className="rounded-lg" value="signin">
                  Sign in
                </TabsTrigger>
                <TabsTrigger className="rounded-lg" value="signup">
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
            <p className="mt-6 text-center text-xs leading-5 text-muted-foreground">
              By continuing, you agree to keep things simple.
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
    <div className="rounded-2xl border bg-card/60 p-4 backdrop-blur">
      <span className="mb-3 grid size-9 place-items-center rounded-xl bg-primary/10 text-primary [&_svg]:size-4">
        {icon}
      </span>
      <p className="font-medium">{title}</p>
      <p className="mt-1 text-sm leading-5 text-muted-foreground">
        {description}
      </p>
    </div>
  );
}

function ErrorMessage({ message }: { message: string }) {
  return (
    <p className="rounded-lg bg-destructive/10 px-3 py-2 text-sm text-destructive">
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
      <Label htmlFor={id} className="text-[13px] font-medium">
        {label}
      </Label>
      <Input
        id={id}
        type={type}
        value={value}
        placeholder={placeholder}
        className="h-11 rounded-xl bg-background px-3"
        onChange={(e: React.ChangeEvent<HTMLInputElement>) =>
          onChange(e.target.value)
        }
        required
      />
    </div>
  );
}
