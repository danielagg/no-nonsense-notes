import { useState } from 'react';
import { useMutation } from '@tanstack/react-query';
import { signup, signin } from '@/lib/api';
import { useAuth } from '@/lib/auth-context';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';

export function AuthPage() {
  const { login } = useAuth();
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');

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
    <div className="min-h-screen flex items-center justify-center p-4">
      <Card className="w-full max-w-sm">
        <CardHeader className="text-center">
          <CardTitle className="text-2xl">No Nonsense Notes</CardTitle>
          <CardDescription>Sandbox — test auth & sync APIs</CardDescription>
        </CardHeader>
        <CardContent>
          <Tabs defaultValue="signin">
            <TabsList className="grid w-full grid-cols-2">
              <TabsTrigger value="signin">Sign In</TabsTrigger>
              <TabsTrigger value="signup">Sign Up</TabsTrigger>
            </TabsList>

            <TabsContent value="signin" className="space-y-4 mt-4">
              <form
                onSubmit={(e) => {
                  e.preventDefault();
                  signinMutation.mutate();
                }}
                className="space-y-4"
              >
                <Field label="Email" value={email} onChange={setEmail} type="email" />
                <Field label="Password" value={password} onChange={setPassword} type="password" />
                {error && <p className="text-sm text-destructive">{error.message}</p>}
                <Button type="submit" className="w-full" disabled={isLoading}>
                  {isLoading ? 'Signing in...' : 'Sign In'}
                </Button>
              </form>
            </TabsContent>

            <TabsContent value="signup" className="space-y-4 mt-4">
              <form
                onSubmit={(e) => {
                  e.preventDefault();
                  signupMutation.mutate();
                }}
                className="space-y-4"
              >
                <Field label="Email" value={email} onChange={setEmail} type="email" />
                <Field label="Password" value={password} onChange={setPassword} type="password" />
                {error && <p className="text-sm text-destructive">{error.message}</p>}
                <Button type="submit" className="w-full" disabled={isLoading}>
                  {isLoading ? 'Creating account...' : 'Sign Up'}
                </Button>
              </form>
            </TabsContent>
          </Tabs>
        </CardContent>
      </Card>
    </div>
  );
}

function Field({
  label,
  value,
  onChange,
  type,
}: {
  label: string;
  value: string;
  onChange: (v: string) => void;
  type: string;
}) {
  return (
    <div className="space-y-1.5">
      <Label>{label}</Label>
      <Input type={type} value={value}                   onChange={(e: React.ChangeEvent<HTMLInputElement>) => onChange(e.target.value)} required />
    </div>
  );
}
