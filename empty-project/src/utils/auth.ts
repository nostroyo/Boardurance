// Authentication utility functions

export interface User {
  uuid: string;
  email: string;
  team_name: string;
  role: string;
}

export interface AuthState {
  user: User | null;
  isAuthenticated: boolean;
  isLoading: boolean;
}

// Global auth state management
let authState: AuthState = {
  user: null,
  isAuthenticated: false,
  isLoading: false
};

// Auth state listeners for reactive updates
type AuthStateListener = (state: AuthState) => void;
const authStateListeners: AuthStateListener[] = [];

export const authUtils = {
  // Get current auth state
  getAuthState(): AuthState {
    return { ...authState };
  },

  // Subscribe to auth state changes
  subscribe(listener: AuthStateListener): () => void {
    authStateListeners.push(listener);
    return () => {
      const index = authStateListeners.indexOf(listener);
      if (index > -1) {
        authStateListeners.splice(index, 1);
      }
    };
  },

  // Update auth state and notify listeners
  updateAuthState(updates: Partial<AuthState>): void {
    authState = { ...authState, ...updates };
    authStateListeners.forEach(listener => listener(authState));
  },

  // Set current user (from successful login/register)
  setCurrentUser(user: User): void {
    this.updateAuthState({
      user,
      isAuthenticated: true,
      isLoading: false
    });
  },

  // Clear current user (logout)
  clearCurrentUser(): void {
    this.updateAuthState({
      user: null,
      isAuthenticated: false,
      isLoading: false
    });
  },

  // Check if user is authenticated (based on current state)
  isAuthenticated(): boolean {
    return authState.isAuthenticated;
  },

  // Get current user
  getCurrentUser(): User | null {
    return authState.user;
  },

  // Update user data (e.g., team name change)
  updateUser(updates: Partial<User>): void {
    if (authState.user) {
      const updatedUser = { ...authState.user, ...updates };
      this.setCurrentUser(updatedUser);
    }
  },

  // Set loading state
  setLoading(isLoading: boolean): void {
    this.updateAuthState({ isLoading });
  },

  // Logout (calls API and clears state)
  async logout(): Promise<void> {
    try {
      await apiUtils.logout();
    } catch (error) {
      console.error('Logout API call failed:', error);
      // Still clear local state even if API call fails
    } finally {
      this.clearCurrentUser();
    }
  }
};

// API utility functions with cookie-based authentication
export const apiUtils = {
  // Base API URL
  baseUrl: 'http://localhost:3000/api/v1',

  // Default fetch options for authenticated requests
  getAuthenticatedFetchOptions(options: RequestInit = {}): RequestInit {
    return {
      ...options,
      credentials: 'include', // Include cookies in requests
      headers: {
        'Content-Type': 'application/json',
        ...options.headers,
      },
    };
  },

  // Handle API responses with automatic token refresh
  async handleAuthenticatedResponse(response: Response): Promise<{ success: boolean; data?: any; error?: string }> {
    if (response.status === 401) {
      // Try to refresh token
      const refreshResult = await this.refreshToken();
      if (refreshResult.success) {
        // Token refreshed successfully, but the original request still failed
        // The caller should retry the request
        return { success: false, error: 'Authentication expired, please retry', shouldRetry: true } as any;
      } else {
        // Refresh failed, user needs to login again
        authUtils.clearCurrentUser();
        return { success: false, error: 'Session expired. Please login again.' };
      }
    }

    try {
      const data = await response.json();
      if (response.ok) {
        return { success: true, data };
      } else {
        return { success: false, error: data.error || `Request failed with status ${response.status}` };
      }
    } catch (error) {
      if (response.ok) {
        return { success: true, data: null };
      } else {
        return { success: false, error: `Request failed with status ${response.status}` };
      }
    }
  },

  // Make authenticated request with automatic retry on token refresh
  async makeAuthenticatedRequest(url: string, options: RequestInit = {}): Promise<{ success: boolean; data?: any; error?: string }> {
    const authOptions = this.getAuthenticatedFetchOptions(options);
    
    try {
      const response = await fetch(url, authOptions);
      const result = await this.handleAuthenticatedResponse(response);
      
      // If we should retry (token was refreshed), try once more
      if (!result.success && (result as any).shouldRetry) {
        const retryResponse = await fetch(url, authOptions);
        return await this.handleAuthenticatedResponse(retryResponse);
      }
      
      return result;
    } catch (error) {
      return { success: false, error: 'Network error. Please check your connection.' };
    }
  },

  // Register new user
  async register(email: string, password: string, teamName: string): Promise<{ success: boolean; data?: any; error?: string }> {
    try {
      const response = await fetch(`${this.baseUrl}/auth/register`, {
        method: 'POST',
        credentials: 'include', // Include cookies for auth tokens
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          email,
          password,
          team_name: teamName
        }),
      });

      const data = await response.json();

      if (response.ok) {
        // Registration successful, user is automatically logged in
        authUtils.setCurrentUser(data.user);
        return { success: true, data };
      } else {
        return { success: false, error: data.error || 'Registration failed' };
      }
    } catch (error) {
      return { success: false, error: 'Network error. Please check your connection.' };
    }
  },

  // Login user
  async login(email: string, password: string): Promise<{ success: boolean; data?: any; error?: string }> {
    try {
      const response = await fetch(`${this.baseUrl}/auth/login`, {
        method: 'POST',
        credentials: 'include', // Include cookies for auth tokens
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          email,
          password
        }),
      });

      const data = await response.json();

      if (response.ok) {
        // Login successful, update auth state
        authUtils.setCurrentUser(data.user);
        return { success: true, data };
      } else {
        return { success: false, error: data.error || 'Login failed' };
      }
    } catch (error) {
      return { success: false, error: 'Network error. Please check your connection.' };
    }
  },

  // Logout user
  async logout(): Promise<{ success: boolean; error?: string }> {
    try {
      const response = await fetch(`${this.baseUrl}/auth/logout`, {
        method: 'POST',
        credentials: 'include', // Include cookies for auth tokens
        headers: {
          'Content-Type': 'application/json',
        },
      });

      if (response.ok) {
        return { success: true };
      } else {
        const data = await response.json();
        return { success: false, error: data.error || 'Logout failed' };
      }
    } catch (error) {
      return { success: false, error: 'Network error during logout.' };
    }
  },

  // Refresh authentication token
  async refreshToken(): Promise<{ success: boolean; error?: string }> {
    try {
      const response = await fetch(`${this.baseUrl}/auth/refresh`, {
        method: 'POST',
        credentials: 'include', // Include cookies for refresh token
        headers: {
          'Content-Type': 'application/json',
        },
      });

      if (response.ok) {
        return { success: true };
      } else {
        const data = await response.json();
        return { success: false, error: data.error || 'Token refresh failed' };
      }
    } catch (error) {
      return { success: false, error: 'Network error during token refresh.' };
    }
  },

  // Get player data by UUID
  async getPlayer(uuid: string): Promise<{ success: boolean; data?: any; error?: string }> {
    return await this.makeAuthenticatedRequest(`${this.baseUrl}/players/${uuid}`, {
      method: 'GET',
    });
  },

  // Update player team name
  async updatePlayerTeamName(uuid: string, teamName: string): Promise<{ success: boolean; data?: any; error?: string }> {
    return await this.makeAuthenticatedRequest(`${this.baseUrl}/players/${uuid}`, {
      method: 'PUT',
      body: JSON.stringify({
        team_name: teamName
      }),
    });
  },

  // Get all players (admin only)
  async getAllPlayers(): Promise<{ success: boolean; data?: any; error?: string }> {
    return await this.makeAuthenticatedRequest(`${this.baseUrl}/players`, {
      method: 'GET',
    });
  },

  // Check authentication status by making a test request
  async checkAuthStatus(): Promise<{ success: boolean; user?: User; error?: string }> {
    try {
      // Try to get current user's data to verify authentication
      const currentUser = authUtils.getCurrentUser();
      if (!currentUser) {
        return { success: false, error: 'No user in local state' };
      }

      const result = await this.getPlayer(currentUser.uuid);
      if (result.success) {
        // Update user data in case it changed on the server
        authUtils.setCurrentUser(result.data);
        return { success: true, user: result.data };
      } else {
        // Authentication failed
        authUtils.clearCurrentUser();
        return { success: false, error: result.error };
      }
    } catch (error) {
      authUtils.clearCurrentUser();
      return { success: false, error: 'Authentication check failed' };
    }
  }
};