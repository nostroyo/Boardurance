import React, { createContext, useContext, useEffect, useState } from 'react';
import { authUtils, apiUtils } from '../utils/auth';
import type { User, AuthState } from '../utils/auth';

interface AuthContextType {
  user: User | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  error: string | null;
  login: (email: string, password: string) => Promise<{ success: boolean; error?: string }>;
  register: (email: string, password: string, teamName: string) => Promise<{ success: boolean; error?: string }>;
  logout: () => Promise<void>;
  updateUser: (updates: Partial<User>) => void;
  checkAuthStatus: () => Promise<void>;
  clearError: () => void;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

export const useAuthContext = () => {
  const context = useContext(AuthContext);
  if (context === undefined) {
    throw new Error('useAuthContext must be used within an AuthProvider');
  }
  return context;
};

interface AuthProviderProps {
  children: React.ReactNode;
}

export const AuthProvider: React.FC<AuthProviderProps> = ({ children }) => {
  const [authState, setAuthState] = useState<AuthState>(authUtils.getAuthState());
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    // Subscribe to auth state changes
    const unsubscribe = authUtils.subscribe((newState) => {
      setAuthState(newState);
    });

    // Force loading to false since we disabled auth check
    authUtils.setLoading(false);

    // Check authentication status on mount
    checkAuthStatus();

    return unsubscribe;
  }, []);

  const login = async (email: string, password: string): Promise<{ success: boolean; error?: string }> => {
    setError(null);
    authUtils.setLoading(true);
    
    try {
      const result = await apiUtils.login(email, password);
      
      if (result.success) {
        return { success: true };
      } else {
        setError(result.error || 'Login failed');
        return { success: false, error: result.error };
      }
    } catch (error) {
      const errorMessage = 'Login failed due to network error';
      setError(errorMessage);
      return { success: false, error: errorMessage };
    } finally {
      authUtils.setLoading(false);
    }
  };

  const register = async (email: string, password: string, teamName: string): Promise<{ success: boolean; error?: string }> => {
    setError(null);
    authUtils.setLoading(true);
    
    try {
      const result = await apiUtils.register(email, password, teamName);
      
      if (result.success) {
        return { success: true };
      } else {
        setError(result.error || 'Registration failed');
        return { success: false, error: result.error };
      }
    } catch (error) {
      const errorMessage = 'Registration failed due to network error';
      setError(errorMessage);
      return { success: false, error: errorMessage };
    } finally {
      authUtils.setLoading(false);
    }
  };

  const logout = async (): Promise<void> => {
    setError(null);
    authUtils.setLoading(true);
    
    try {
      await authUtils.logout();
    } catch (error) {
      console.error('Logout error:', error);
      // Still clear local state even if API call fails
      authUtils.clearCurrentUser();
    } finally {
      authUtils.setLoading(false);
    }
  };

  const updateUser = (updates: Partial<User>): void => {
    authUtils.updateUser(updates);
  };

  const checkAuthStatus = async (): Promise<void> => {
    console.log('Starting auth status check...');
    setError(null);
    authUtils.setLoading(true);
    
    try {
      const result = await apiUtils.checkAuthStatus();
      console.log('Auth status check result:', result);
      
      if (!result.success) {
        // Clear auth state if check failed
        authUtils.clearCurrentUser();
        if (result.error && !result.error.includes('No user in local state')) {
          setError(result.error);
        }
      }
      // If successful, user state is already updated by checkAuthStatus
    } catch (error) {
      console.error('Auth status check failed:', error);
      authUtils.clearCurrentUser();
      setError('Authentication check failed');
    } finally {
      console.log('Auth status check completed, setting loading to false');
      authUtils.setLoading(false);
    }
  };

  const clearError = (): void => {
    setError(null);
  };

  const value: AuthContextType = {
    user: authState.user,
    isAuthenticated: authState.isAuthenticated,
    isLoading: authState.isLoading,
    error,
    login,
    register,
    logout,
    updateUser,
    checkAuthStatus,
    clearError,
  };

  return (
    <AuthContext.Provider value={value}>
      {children}
    </AuthContext.Provider>
  );
};