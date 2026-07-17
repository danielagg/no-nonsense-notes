import { ArrowRight } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";

interface Props {
  email: string;
  password: string;
  error?: Error | null;
  isLoading: boolean;
  onEmailChange: (value: string) => void;
  onPasswordChange: (value: string) => void;
  onSignin: () => void;
  onSignup: () => void;
}

export function AuthCard(props: Props) {
  return (
    <Card className="terminal-glow mx-auto w-full max-w-md gap-0 rounded-lg border border-primary/20 bg-card/92 py-0 ring-0 backdrop-blur">
      <CardHeader className="px-6 pb-6 pt-7 sm:px-8 sm:pt-8">
        <p className="mb-3 font-heading text-[10px] font-semibold uppercase tracking-[0.16em] text-primary">Auth // workspace access</p>
        <CardTitle className="font-heading text-2xl font-semibold tracking-[-0.04em]">Identify yourself</CardTitle>
        <CardDescription className="mt-1">Sign in or initialize a new account.</CardDescription>
      </CardHeader>
      <CardContent className="px-6 pb-7 sm:px-8 sm:pb-8">
        <Tabs defaultValue="signin">
          <TabsList className="grid h-10 w-full grid-cols-2 rounded-md border border-primary/10 bg-muted/65 p-1">
            <TabsTrigger className="rounded-sm font-heading text-xs" value="signin">Sign in</TabsTrigger>
            <TabsTrigger className="rounded-sm font-heading text-xs" value="signup">Create account</TabsTrigger>
          </TabsList>
          <AuthForm mode="signin" {...props} />
          <AuthForm mode="signup" {...props} />
        </Tabs>
        <p className="mt-6 text-center font-heading text-[10px] uppercase tracking-[0.08em] text-muted-foreground">Encrypted transport // local-first data</p>
      </CardContent>
    </Card>
  );
}

function AuthForm({ mode, ...props }: Props & { mode: "signin" | "signup" }) {
  const isSignin = mode === "signin";
  return (
    <TabsContent value={mode} className="mt-6">
      <form onSubmit={(event) => { event.preventDefault(); (isSignin ? props.onSignin : props.onSignup)(); }} className="space-y-5">
        <Field id={`${mode}-email`} label="Email address" value={props.email} onChange={props.onEmailChange} type="email" placeholder="you@company.com" />
        <Field id={`${mode}-password`} label="Password" value={props.password} onChange={props.onPasswordChange} type="password" placeholder={isSignin ? "Enter your password" : "Choose a secure password"} />
        {props.error && <p className="rounded-sm border border-destructive/25 bg-destructive/10 px-3 py-2 font-mono text-xs text-destructive">{props.error.message}</p>}
        <Button type="submit" size="lg" className="w-full" disabled={props.isLoading}>
          {props.isLoading ? (isSignin ? "Signing in..." : "Creating account...") : (isSignin ? "Sign in" : "Create account")}
          {!props.isLoading && <ArrowRight data-icon="inline-end" />}
        </Button>
      </form>
    </TabsContent>
  );
}

function Field({ id, label, value, onChange, type, placeholder }: { id: string; label: string; value: string; onChange: (value: string) => void; type: string; placeholder: string }) {
  return (
    <div className="space-y-2">
      <Label htmlFor={id} className="font-heading text-[11px] font-semibold uppercase tracking-[0.08em] text-muted-foreground">{label}</Label>
      <Input id={id} type={type} value={value} placeholder={placeholder} className="h-11 rounded-md bg-background/60 px-3 font-mono" onChange={(event) => onChange(event.target.value)} required />
    </div>
  );
}
