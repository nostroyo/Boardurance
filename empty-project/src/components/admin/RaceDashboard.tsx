import React, { useState, useEffect } from 'react';
import { raceAPI } from '../../utils/raceAPI';
import RaceCard from './RaceCard';

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

type RaceFilter = 'all' | 'waiting' | 'in-progress' | 'finished' | 'cancelled';

const RaceDashboard: React.FC = () => {
  const [races, setRaces] = useState<Race[]>([]);
  const [filteredRaces, setFilteredRaces] = useState<Race[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [filter, setFilter] = useState<RaceFilter>('all');
  const [searchTerm, setSearchTerm] = useState('');

  const loadRaces = async () => {
    setIsLoading(true);
    setError(null);
    
    try {
      const result = await raceAPI.getAllRaces();
      
      if (result.success) {
        setRaces(result.data || []);
      } else {
        setError(result.error || 'Failed to load races');
      }
    } catch (error) {
      setError('Network error. Please try again.');
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    loadRaces();
  }, []);

  useEffect(() => {
    let filtered = races;

    // Apply status filter
    if (filter !== 'all') {
      const statusMap: Record<RaceFilter, string> = {
        'waiting': 'Waiting',
        'in-progress': 'InProgress',
        'finished': 'Finished',
        'cancelled': 'Cancelled',
        'all': ''
      };
      filtered = filtered.filter(race => race.status === statusMap[filter]);
    }

    // Apply search filter
    if (searchTerm.trim()) {
      const term = searchTerm.toLowerCase();
      filtered = filtered.filter(race => 
        race.name.toLowerCase().includes(term) ||
        race.track.name.toLowerCase().includes(term)
      );
    }

    setFilteredRaces(filtered);
  }, [races, filter, searchTerm]);

  const getStatusCounts = () => {
    return {
      all: races.length,
      waiting: races.filter(r => r.status === 'Waiting').length,
      'in-progress': races.filter(r => r.status === 'InProgress').length,
      finished: races.filter(r => r.status === 'Finished').length,
      cancelled: races.filter(r => r.status === 'Cancelled').length,
    };
  };

  const statusCounts = getStatusCounts();

  const handleRaceAction = async (raceUuid: string, action: 'start' | 'view') => {
    if (action === 'start') {
      try {
        const result = await raceAPI.startRace(raceUuid);
        if (result.success) {
          // Refresh races to show updated status
          await loadRaces();
        } else {
          setError(result.error || 'Failed to start race');
        }
      } catch (error) {
        setError('Network error. Please try again.');
      }
    } else if (action === 'view') {
      // TODO: Implement race details view
      console.log('View race details:', raceUuid);
    }
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-spin rounded-full h-32 w-32 border-b-2 border-blue-500"></div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex justify-between items-center">
        <h2 className="text-2xl font-bold text-gray-900">Race Management</h2>
        <button
          onClick={loadRaces}
          className="inline-flex items-center px-4 py-2 border border-gray-300 shadow-sm text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
        >
          <svg className="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
          </svg>
          Refresh
        </button>
      </div>

      {/* Error Display */}
      {error && (
        <div className="bg-red-50 border border-red-200 rounded-md p-4">
          <div className="flex">
            <div className="flex-shrink-0">
              <svg className="h-5 w-5 text-red-400" viewBox="0 0 20 20" fill="currentColor">
                <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clipRule="evenodd" />
              </svg>
            </div>
            <div className="ml-3">
              <p className="text-sm text-red-800">{error}</p>
            </div>
            <div className="ml-auto pl-3">
              <button
                onClick={() => setError(null)}
                className="inline-flex text-red-400 hover:text-red-600"
              >
                <svg className="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                  <path fillRule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clipRule="evenodd" />
                </svg>
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Filters and Search */}
      <div className="bg-white shadow rounded-lg p-6">
        <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between space-y-4 sm:space-y-0">
          {/* Status Filters */}
          <div className="flex flex-wrap gap-2">
            {Object.entries(statusCounts).map(([status, count]) => (
              <button
                key={status}
                onClick={() => setFilter(status as RaceFilter)}
                className={`inline-flex items-center px-3 py-1 rounded-full text-sm font-medium ${
                  filter === status
                    ? 'bg-blue-100 text-blue-800'
                    : 'bg-gray-100 text-gray-700 hover:bg-gray-200'
                }`}
              >
                {status.charAt(0).toUpperCase() + status.slice(1).replace('-', ' ')}
                <span className="ml-1 bg-white rounded-full px-2 py-0.5 text-xs">
                  {count}
                </span>
              </button>
            ))}
          </div>

          {/* Search */}
          <div className="relative">
            <div className="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
              <svg className="h-5 w-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
              </svg>
            </div>
            <input
              type="text"
              placeholder="Search races or tracks..."
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
              className="block w-full pl-10 pr-3 py-2 border border-gray-300 rounded-md leading-5 bg-white placeholder-gray-500 focus:outline-none focus:placeholder-gray-400 focus:ring-1 focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
            />
          </div>
        </div>
      </div>

      {/* Race List */}
      {filteredRaces.length === 0 ? (
        <div className="text-center py-12">
          <svg className="mx-auto h-12 w-12 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5H7a2 2 0 00-2 2v10a2 2 0 002 2h8a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2" />
          </svg>
          <h3 className="mt-2 text-sm font-medium text-gray-900">No races found</h3>
          <p className="mt-1 text-sm text-gray-500">
            {races.length === 0 
              ? "Get started by creating your first race."
              : "Try adjusting your filters or search terms."
            }
          </p>
        </div>
      ) : (
        <div className="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3">
          {filteredRaces.map((race) => (
            <RaceCard
              key={race.uuid}
              race={race}
              onAction={handleRaceAction}
            />
          ))}
        </div>
      )}
    </div>
  );
};

export default RaceDashboard;