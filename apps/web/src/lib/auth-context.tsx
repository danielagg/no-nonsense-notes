import { createContext, useContext, useState, useCallback, type ReactNode } from 'react';
import { clearAll } from './loro-store';

interface AuthState {
  token: string | null;
  accountId: string | null;
  login: (token: string, accountId: string) => void;
  logout: () => void;
  isAuthenticated: boolean;
}

const AuthContext = createContext<AuthState | null>(null);

const TOKEN_KEY = 'nnn-token';
const ACCOUNT_KEY = 'nnn-account';

export function AuthProvider({ children }: { children: ReactNode }) {
  const [token, setToken] = useState<string | null>(() => localStorage.getItem(TOKEN_KEY));
  const [accountId, setAccountId] = useState<string | null>(() => localStorage.getItem(ACCOUNT_KEY));

  const login = useCallback((t: string, id: string) => {
    localStorage.setItem(TOKEN_KEY, t);
    localStorage.setItem(ACCOUNT_KEY, id);
    setToken(t);
    setAccountId(id);
  }, []);

  const logout = useCallback(() => {
    localStorage.removeItem(TOKEN_KEY);
    localStorage.removeItem(ACCOUNT_KEY);
    clearAll();
    setToken(null);
    setAccountId(null);
  }, []);

  return (
    <AuthContext.Provider value={{ token, accountId, login, logout, isAuthenticated: !!token }}>
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth() {
  const ctx = useContext(AuthContext);
  if (!ctx) throw new Error('useAuth must be used within AuthProvider');
  return ctx;
}
