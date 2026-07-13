import { useState, useCallback, type ReactNode } from 'react';
import { AuthContext } from './auth';
import { setActiveAccount } from './wasm';

const TOKEN_KEY = 'nnn-token';
const ACCOUNT_KEY = 'nnn-account';

export function AuthProvider({ children }: { children: ReactNode }) {
  const [token, setToken] = useState<string | null>(() => localStorage.getItem(TOKEN_KEY));
  const [accountId, setAccountId] = useState<string | null>(() => localStorage.getItem(ACCOUNT_KEY));

  const login = useCallback((t: string, id: string) => {
    localStorage.setItem(TOKEN_KEY, t);
    localStorage.setItem(ACCOUNT_KEY, id);
    setActiveAccount(id);
    setToken(t);
    setAccountId(id);
  }, []);

  const logout = useCallback(() => {
    setActiveAccount(null);
    localStorage.removeItem(TOKEN_KEY);
    localStorage.removeItem(ACCOUNT_KEY);
    setToken(null);
    setAccountId(null);
  }, []);

  return (
    <AuthContext.Provider value={{ token, accountId, login, logout, isAuthenticated: !!token }}>
      {children}
    </AuthContext.Provider>
  );
}
