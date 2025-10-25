// Authentication utility functions

export interface User {
  uuid: string;
  email: string;
  team_name: string;
}

export const AUTH_STORAGE_KEY = 'currentUser';

export const authUtils = {
  // Get current user from localStorage
  getCurrentUser(): User | null {
    try {
      const stored = localStorage.getItem(AUTH_STORAGE_KEY);
      return stored ? JSON.parse(stored) : null;
    } catch (error) {
      console.error('Error parsing stored user data:', error);
      return null;
    }
  },

  // Set current user in localStorage
  setCurrentUser(user: User): void {
    localStorage.setItem(AUTH_STORAGE_KEY, JSON.stringify(user));
  },

  // Remove current user from localStorage
  logout(): void {
    localStorage.removeItem(AUTH_STORAGE_KEY);
  },

  // Check if user is authenticated
  isAuthenticated(): boolean {
    return this.getCurrentUser() !== null;
  },

  // Update user data (e.g., team name change)
  updateUser(updates: Partial<User>): void {
    const currentUser = this.getCurrentUser();
    if (currentUser) {
      const updatedUser = { ...currentUser, ...updates };
      this.setCurrentUser(updatedUser);
    }
  }
};

// API utility functions
export const apiUtils = {
  // Base API URL
  baseUrl: 'http://localhost:3000/api/v1',

  // Register new user
  async register(email: string, password: string, teamName: string): Promise<{ success: boolean; data?: any; error?: string }> {
    try {
      const response = await fetch(`${this.baseUrl}/auth/register`, {
        method: 'POST',
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
        return { success: true, data };
      } else {
        return { success: false, error: data.error || 'Login failed' };
      }
    } catch (error) {
      return { success: false, error: 'Network error. Please check your connection.' };
    }
  },

  // Get player data by UUID
  async getPlayer(uuid: string): Promise<{ success: boolean; data?: any; error?: string }> {
    try {
      const response = await fetch(`${this.baseUrl}/players/${uuid}`, {
        method: 'GET',
        headers: {
          'Content-Type': 'application/json',
        },
      });

      if (response.ok) {
        const data = await response.json();
        return { success: true, data };
      } else {
        return { success: false, error: 'Failed to load player data' };
      }
    } catch (error) {
      return { success: false, error: 'Network error. Please check your connection.' };
    }
  },

  // Update player team name
  async updatePlayerTeamName(uuid: string, teamName: string): Promise<{ success: boolean; data?: any; error?: string }> {
    try {
      const response = await fetch(`${this.baseUrl}/players/${uuid}`, {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          team_name: teamName
        }),
      });

      if (response.ok) {
        const data = await response.json();
        return { success: true, data };
      } else {
        return { success: false, error: 'Failed to update team name' };
      }
    } catch (error) {
      return { success: false, error: 'Network error. Please check your connection.' };
    }
  }
};