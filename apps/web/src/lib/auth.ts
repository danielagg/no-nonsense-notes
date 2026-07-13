import { createContext, useContext } from 'react';

export interface AuthState {
  token: string | null;
  accountId: string | null;
  login: (token: string, accountId: string) => void;
  logout: () => void;
  isAuthenticated: boolean;
}

export const AuthContext = createContext<AuthState | null>(null);

export function useAuth() {
  const context = useContext(AuthContext);
  if (!context) throw new Error('useAuth must be used within AuthProvider');
  return context;
}
