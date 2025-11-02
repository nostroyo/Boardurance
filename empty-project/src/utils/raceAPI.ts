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

interface APIResponse<T = any> {
  success: boolean;
  data?: T;
  error?: string;
}

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
          error: data.error || data.message || `Request failed with status ${response.status}` 
        };
      }
    } catch (error) {
      if (response.ok) {
        return { success: true, data: null as T };
      } else {
        return { 
          success: false, 
          error: `Request failed with status ${response.status}` 
        };
      }
    }
  },

  // Make authenticated request
  async makeAuthenticatedRequest<T>(url: string, options: RequestInit = {}): Promise<APIResponse<T>> {
    const authOptions = this.getAuthenticatedFetchOptions(options);
    
    try {
      const response = await fetch(url, authOptions);
      return await this.handleResponse<T>(response);
    } catch (error) {
      return { 
        success: false, 
        error: 'Network error. Please check your connection.' 
      };
    }
  },

  // Create new race
  async createRace(raceData: CreateRaceRequest): Promise<APIResponse<RaceResponse>> {
    return await this.makeAuthenticatedRequest<RaceResponse>(
      `${this.baseUrl}/races`,
      {
        method: 'POST',
        body: JSON.stringify(raceData),
      }
    );
  },

  // Get all races
  async getAllRaces(): Promise<APIResponse<Race[]>> {
    return await this.makeAuthenticatedRequest<Race[]>(
      `${this.baseUrl}/races`,
      {
        method: 'GET',
      }
    );
  },

  // Get specific race
  async getRace(raceUuid: string): Promise<APIResponse<Race>> {
    return await this.makeAuthenticatedRequest<Race>(
      `${this.baseUrl}/races/${raceUuid}`,
      {
        method: 'GET',
      }
    );
  },

  // Get race status
  async getRaceStatus(raceUuid: string): Promise<APIResponse<string>> {
    return await this.makeAuthenticatedRequest<string>(
      `${this.baseUrl}/races/${raceUuid}/status`,
      {
        method: 'GET',
      }
    );
  },

  // Start race
  async startRace(raceUuid: string): Promise<APIResponse<RaceResponse>> {
    return await this.makeAuthenticatedRequest<RaceResponse>(
      `${this.baseUrl}/races/${raceUuid}/start`,
      {
        method: 'POST',
      }
    );
  },

  // Join race
  async joinRace(raceUuid: string, playerUuid: string, carUuid: string, pilotUuid: string): Promise<APIResponse<RaceResponse>> {
    return await this.makeAuthenticatedRequest<RaceResponse>(
      `${this.baseUrl}/races/${raceUuid}/join`,
      {
        method: 'POST',
        body: JSON.stringify({
          player_uuid: playerUuid,
          car_uuid: carUuid,
          pilot_uuid: pilotUuid,
        }),
      }
    );
  },

  // Process race turn
  async processRaceTurn(raceUuid: string, actions: Array<{ player_uuid: string; boost_value: number }>): Promise<APIResponse<any>> {
    return await this.makeAuthenticatedRequest(
      `${this.baseUrl}/races/${raceUuid}/turn`,
      {
        method: 'POST',
        body: JSON.stringify({
          actions: actions,
        }),
      }
    );
  },
};