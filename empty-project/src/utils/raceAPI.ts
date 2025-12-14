// Race API utility functions

interface CreateSectorRequest {
  id: number;
  name: string;
  min_value: number;
  max_value: number;
  slot_capacity: number | null;
  sector_type: 'Start' | 'Straight' | 'Curve' | 'Finish';
}

interface CreateRaceRequest {
  name: string;
  track_name: string;
  total_laps: number;
  sectors: CreateSectorRequest[];
}

interface Race {
  uuid: string;
  name: string;
  track: {
    uuid: string;
    name: string;
    sectors: Array<{
      id: number;
      name: string;
      min_value: number;
      max_value: number;
      slot_capacity: number | null;
      sector_type: string;
    }>;
  };
  participants: Array<{
    player_uuid: string;
    car_uuid: string;
    pilot_uuid: string;
    current_sector: number;
    current_position_in_sector: number;
    current_lap: number;
    total_value: number;
    is_finished: boolean;
    finish_position: number | null;
  }>;
  lap_characteristic: string;
  current_lap: number;
  total_laps: number;
  status: 'Waiting' | 'InProgress' | 'Finished' | 'Cancelled';
  created_at: string;
  updated_at: string;
}

interface RaceResponse {
  race: Race;
  message: string;
}

interface APIResponse<T = unknown> {
  success: boolean;
  data?: T;
  error?: string;
}

// Turn phase types for player game interface
export type TurnPhase = 'WaitingForPlayers' | 'AllSubmitted' | 'Processing' | 'Complete';

// Race polling configuration
export interface RacePollingConfig {
  interval: number; // milliseconds
  maxRetries: number;
  retryDelay: number; // base delay for exponential backoff
}

// Default polling configuration
export const DEFAULT_POLLING_CONFIG: RacePollingConfig = {
  interval: 2000, // 2 seconds
  maxRetries: 3,
  retryDelay: 1000, // 1 second base delay
};

export const raceAPI = {
  // Base API URL
  baseUrl: 'http://localhost:3000/api/v1',

  // Default fetch options for authenticated requests
  getAuthenticatedFetchOptions(options: RequestInit = {}): RequestInit {
    return {
      ...options,
      // credentials: 'include', // Include cookies in requests - temporarily disabled for CORS testing
      headers: {
        'Content-Type': 'application/json',
        ...options.headers,
      },
    };
  },

  // Handle API responses
  async handleResponse<T>(response: Response): Promise<APIResponse<T>> {
    try {
      const data = await response.json();

      if (response.ok) {
        return { success: true, data };
      } else {
        return {
          success: false,
          error: data.error || data.message || `Request failed with status ${response.status}`,
        };
      }
    } catch {
      if (response.ok) {
        return { success: true, data: null as T };
      } else {
        return {
          success: false,
          error: `Request failed with status ${response.status}`,
        };
      }
    }
  },

  // Make authenticated request
  async makeAuthenticatedRequest<T>(
    url: string,
    options: RequestInit = {},
  ): Promise<APIResponse<T>> {
    const authOptions = this.getAuthenticatedFetchOptions(options);

    try {
      const response = await fetch(url, authOptions);
      return await this.handleResponse<T>(response);
    } catch {
      return {
        success: false,
        error: 'Network error. Please check your connection.',
      };
    }
  },

  // Create new race
  async createRace(raceData: CreateRaceRequest): Promise<APIResponse<RaceResponse>> {
    return await this.makeAuthenticatedRequest<RaceResponse>(`${this.baseUrl}/races`, {
      method: 'POST',
      body: JSON.stringify(raceData),
    });
  },

  // Get all races
  async getAllRaces(): Promise<APIResponse<Race[]>> {
    return await this.makeAuthenticatedRequest<Race[]>(`${this.baseUrl}/races`, {
      method: 'GET',
    });
  },

  // Get specific race
  async getRace(raceUuid: string): Promise<APIResponse<Race>> {
    return await this.makeAuthenticatedRequest<Race>(`${this.baseUrl}/races/${raceUuid}`, {
      method: 'GET',
    });
  },

  // Get race status
  async getRaceStatus(raceUuid: string): Promise<APIResponse<string>> {
    return await this.makeAuthenticatedRequest<string>(`${this.baseUrl}/races/${raceUuid}/status`, {
      method: 'GET',
    });
  },

  // Start race
  async startRace(raceUuid: string): Promise<APIResponse<RaceResponse>> {
    return await this.makeAuthenticatedRequest<RaceResponse>(
      `${this.baseUrl}/races/${raceUuid}/start`,
      {
        method: 'POST',
      },
    );
  },

  // Join race
  async joinRace(
    raceUuid: string,
    playerUuid: string,
    carUuid: string,
    pilotUuid: string,
  ): Promise<APIResponse<RaceResponse>> {
    return await this.makeAuthenticatedRequest<RaceResponse>(
      `${this.baseUrl}/races/${raceUuid}/join`,
      {
        method: 'POST',
        body: JSON.stringify({
          player_uuid: playerUuid,
          car_uuid: carUuid,
          pilot_uuid: pilotUuid,
        }),
      },
    );
  },

  // Process race turn
  async processRaceTurn(
    raceUuid: string,
    actions: Array<{ player_uuid: string; boost_value: number }>,
  ): Promise<APIResponse<RaceResponse>> {
    return await this.makeAuthenticatedRequest(`${this.baseUrl}/races/${raceUuid}/turn`, {
      method: 'POST',
      body: JSON.stringify({
        actions: actions,
      }),
    });
  },

  // Player Game Interface specific API methods

  // Submit boost action for a player
  async submitBoostAction(
    raceUuid: string,
    playerUuid: string,
    boostValue: number,
  ): Promise<APIResponse<RaceResponse>> {
    if (boostValue < 0 || boostValue > 5) {
      return {
        success: false,
        error: 'Boost value must be between 0 and 5',
      };
    }

    return await this.makeAuthenticatedRequest(`${this.baseUrl}/races/${raceUuid}/boost`, {
      method: 'POST',
      body: JSON.stringify({
        player_uuid: playerUuid,
        boost_value: boostValue,
      }),
    });
  },

  // Get current turn phase for a race
  async getTurnPhase(raceUuid: string): Promise<APIResponse<string>> {
    return await this.makeAuthenticatedRequest<string>(
      `${this.baseUrl}/races/${raceUuid}/turn-phase`,
      {
        method: 'GET',
      },
    );
  },

  // Real-time race polling with error handling and retry logic
  async pollRaceData(raceUuid: string, retryCount: number = 0): Promise<APIResponse<Race>> {
    const maxRetries = 3;
    const retryDelay = 1000 * Math.pow(2, retryCount); // Exponential backoff

    try {
      const response = await this.getRace(raceUuid);

      if (!response.success && retryCount < maxRetries) {
        // Wait before retrying
        await new Promise((resolve) => setTimeout(resolve, retryDelay));
        return await this.pollRaceData(raceUuid, retryCount + 1);
      }

      return response;
    } catch {
      if (retryCount < maxRetries) {
        await new Promise((resolve) => setTimeout(resolve, retryDelay));
        return await this.pollRaceData(raceUuid, retryCount + 1);
      }

      return {
        success: false,
        error: 'Failed to fetch race data after multiple attempts',
      };
    }
  },

  // Start real-time polling for race updates
  startRacePolling(
    raceUuid: string,
    onUpdate: (race: Race) => void,
    onError: (error: string) => void,
  ): () => void {
    let isPolling = true;
    let pollCount = 0;

    const poll = async () => {
      if (!isPolling) return;

      try {
        const response = await this.pollRaceData(raceUuid);

        if (response.success && response.data) {
          onUpdate(response.data);
          pollCount = 0; // Reset error count on success
        } else {
          pollCount++;
          if (pollCount >= 3) {
            onError(response.error || 'Failed to fetch race data');
            return;
          }
        }
      } catch {
        pollCount++;
        if (pollCount >= 3) {
          onError('Network error during race polling');
          return;
        }
      }

      // Schedule next poll (2-second intervals)
      if (isPolling) {
        setTimeout(poll, 2000);
      }
    };

    // Start polling
    poll();

    // Return cleanup function
    return () => {
      isPolling = false;
    };
  },

  // Get player's performance breakdown for current lap
  async getPlayerPerformance(raceUuid: string, playerUuid: string): Promise<APIResponse<unknown>> {
    return await this.makeAuthenticatedRequest(
      `${this.baseUrl}/races/${raceUuid}/players/${playerUuid}/performance`,
      {
        method: 'GET',
      },
    );
  },

  // Get race history for a player
  async getPlayerRaceHistory(raceUuid: string, playerUuid: string): Promise<APIResponse<unknown>> {
    return await this.makeAuthenticatedRequest(
      `${this.baseUrl}/races/${raceUuid}/players/${playerUuid}/history`,
      {
        method: 'GET',
      },
    );
  },

  // Check if player has submitted action for current turn
  async hasPlayerSubmitted(raceUuid: string, playerUuid: string): Promise<APIResponse<boolean>> {
    return await this.makeAuthenticatedRequest<boolean>(
      `${this.baseUrl}/races/${raceUuid}/players/${playerUuid}/submitted`,
      {
        method: 'GET',
      },
    );
  },

  // Get remaining time for current turn phase
  async getTurnTimeRemaining(raceUuid: string): Promise<APIResponse<number>> {
    return await this.makeAuthenticatedRequest<number>(
      `${this.baseUrl}/races/${raceUuid}/turn-time-remaining`,
      {
        method: 'GET',
      },
    );
  },
};

// Utility functions for race status monitoring and turn phase detection

export const raceStatusUtils = {
  // Check if race is in a state where players can submit actions
  canSubmitActions(race: Race): boolean {
    return race.status === 'InProgress';
  },

  // Check if race is waiting for player actions
  isWaitingForPlayers(turnPhase: TurnPhase): boolean {
    return turnPhase === 'WaitingForPlayers';
  },

  // Check if all players have submitted their actions
  allPlayersSubmitted(turnPhase: TurnPhase): boolean {
    return turnPhase === 'AllSubmitted';
  },

  // Check if race is currently processing a turn
  isProcessingTurn(turnPhase: TurnPhase): boolean {
    return turnPhase === 'Processing';
  },

  // Check if turn is complete and ready for next phase
  isTurnComplete(turnPhase: TurnPhase): boolean {
    return turnPhase === 'Complete';
  },

  // Get user-friendly status message
  getStatusMessage(race: Race, turnPhase: TurnPhase, hasSubmitted: boolean): string {
    if (race.status === 'Waiting') {
      return 'Race is waiting to start';
    }

    if (race.status === 'Finished') {
      return 'Race has finished';
    }

    if (race.status === 'Cancelled') {
      return 'Race has been cancelled';
    }

    // Race is in progress
    switch (turnPhase) {
      case 'WaitingForPlayers':
        return hasSubmitted
          ? 'Waiting for other players to submit their actions'
          : 'Submit your boost action for this lap';
      case 'AllSubmitted':
        return 'All players have submitted. Processing turn...';
      case 'Processing':
        return 'Processing lap results...';
      case 'Complete':
        return 'Turn complete. Preparing next lap...';
      default:
        return 'Unknown race state';
    }
  },

  // Get appropriate UI color for turn phase
  getTurnPhaseColor(turnPhase: TurnPhase): string {
    const colors: Record<TurnPhase, string> = {
      WaitingForPlayers: '#10B981', // Green
      AllSubmitted: '#F59E0B', // Yellow
      Processing: '#3B82F6', // Blue
      Complete: '#6B7280', // Gray
    };
    return colors[turnPhase] || '#6B7280';
  },

  // Validate boost value
  isValidBoostValue(boost: number): boolean {
    return Number.isInteger(boost) && boost >= 0 && boost <= 5;
  },

  // Calculate progress percentage for current lap
  getLapProgress(race: Race): number {
    if (race.total_laps === 0) return 0;
    return Math.min((race.current_lap / race.total_laps) * 100, 100);
  },

  // Check if race is on final lap
  isFinalLap(race: Race): boolean {
    return race.current_lap >= race.total_laps;
  },

  // Get lap characteristic icon
  getLapCharacteristicIcon(characteristic: string): string {
    const icons: Record<string, string> = {
      Straight: '🏁',
      Curve: '🌀',
    };
    return icons[characteristic] || '🏁';
  },
};

// Error handling utilities for race API
export const raceErrorUtils = {
  // Check if error is retryable
  isRetryableError(error: string): boolean {
    const retryableErrors = [
      'Network error',
      'Connection timeout',
      'Server temporarily unavailable',
      'Rate limit exceeded',
    ];
    return retryableErrors.some((retryableError) =>
      error.toLowerCase().includes(retryableError.toLowerCase()),
    );
  },

  // Get user-friendly error message
  getUserFriendlyError(error: string): string {
    const errorMappings: Record<string, string> = {
      'Network error': 'Connection lost. Please check your internet connection.',
      'Race not found': 'This race no longer exists or has been removed.',
      'Player not in race': 'You are not registered for this race.',
      'Invalid boost value': 'Boost value must be between 0 and 5.',
      'Turn phase mismatch': 'Race state has changed. Refreshing...',
      'Action already submitted': 'You have already submitted your action for this turn.',
      'Race not in progress': 'This race is not currently active.',
    };

    // Check for exact matches first
    if (errorMappings[error]) {
      return errorMappings[error];
    }

    // Check for partial matches
    for (const [key, message] of Object.entries(errorMappings)) {
      if (error.toLowerCase().includes(key.toLowerCase())) {
        return message;
      }
    }

    // Default fallback
    return 'An unexpected error occurred. Please try again.';
  },

  // Determine if error requires user action
  requiresUserAction(error: string): boolean {
    const userActionErrors = [
      'Player not in race',
      'Invalid boost value',
      'Action already submitted',
      'Race not in progress',
    ];
    return userActionErrors.some((actionError) =>
      error.toLowerCase().includes(actionError.toLowerCase()),
    );
  },
};
