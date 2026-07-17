import { useState } from "react";
import { useMutation, useQuery } from "@tanstack/react-query";
import { signin, signup } from "@/lib/api";
import { useAuth } from "@/lib/auth";
import { AuthCard } from "./auth-page/auth-card";
import { AuthHeader } from "./auth-page/auth-header";
import { MarketingPanel } from "./auth-page/marketing-panel";

const GITHUB_REPO_API_URL =
  "https://api.github.com/repos/danielagg/no-nonsense-notes";

async function getGithubStarCount() {
  const response = await fetch(GITHUB_REPO_API_URL);
  if (!response.ok) throw new Error("Could not load GitHub stars");
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
      <AuthHeader githubStarCount={githubStarCount} />
      <main className="relative z-10 mx-auto grid min-h-[calc(100svh-5rem)] w-full max-w-7xl items-center gap-16 px-5 pb-12 sm:px-8 lg:grid-cols-[1fr_460px] lg:pb-20">
        <MarketingPanel />
        <AuthCard
          email={email}
          password={password}
          error={signupMutation.error || signinMutation.error}
          isLoading={signupMutation.isPending || signinMutation.isPending}
          onEmailChange={setEmail}
          onPasswordChange={setPassword}
          onSignin={() => signinMutation.mutate()}
          onSignup={() => signupMutation.mutate()}
        />
      </main>
    </div>
  );
}
