import React from 'react';

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

interface RaceCardProps {
  race: Race;
  onAction: (raceUuid: string, action: 'start' | 'view') => void;
}

const RaceCard: React.FC<RaceCardProps> = ({ race, onAction }) => {
  const getStatusColor = (status: string) => {
    switch (status) {
      case 'Waiting':
        return 'bg-yellow-100 text-yellow-800';
      case 'InProgress':
        return 'bg-blue-100 text-blue-800';
      case 'Finished':
        return 'bg-green-100 text-green-800';
      case 'Cancelled':
        return 'bg-red-100 text-red-800';
      default:
        return 'bg-gray-100 text-gray-800';
    }
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'Waiting':
        return (
          <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"
            />
          </svg>
        );
      case 'InProgress':
        return (
          <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M14.828 14.828a4 4 0 01-5.656 0M9 10h1.586a1 1 0 01.707.293l2.414 2.414a1 1 0 00.707.293H15M9 10V9a2 2 0 012-2h2a2 2 0 012 2v1M9 10v5a2 2 0 002 2h2a2 2 0 002-2v-5"
            />
          </svg>
        );
      case 'Finished':
        return (
          <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
            />
          </svg>
        );
      case 'Cancelled':
        return (
          <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z"
            />
          </svg>
        );
      default:
        return null;
    }
  };

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  const getProgressPercentage = () => {
    if (race.status === 'Finished') return 100;
    if (race.status === 'Waiting') return 0;
    return Math.round((race.current_lap / race.total_laps) * 100);
  };

  return (
    <div className="bg-white overflow-hidden shadow rounded-lg hover:shadow-md transition-shadow">
      <div className="p-6">
        {/* Header */}
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-medium text-gray-900 truncate" title={race.name}>
            {race.name}
          </h3>
          <span
            className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${getStatusColor(race.status)}`}
          >
            {getStatusIcon(race.status)}
            <span className="ml-1">{race.status}</span>
          </span>
        </div>

        {/* Track Info */}
        <div className="mb-4">
          <p className="text-sm text-gray-600 mb-1">
            <span className="font-medium">Track:</span> {race.track.name}
          </p>
          <p className="text-sm text-gray-600">
            <span className="font-medium">Sectors:</span> {race.track.sectors.length}
          </p>
        </div>

        {/* Race Progress */}
        <div className="mb-4">
          <div className="flex justify-between text-sm text-gray-600 mb-1">
            <span>Progress</span>
            <span>
              {race.current_lap} / {race.total_laps} laps
            </span>
          </div>
          <div className="w-full bg-gray-200 rounded-full h-2">
            <div
              className="bg-blue-600 h-2 rounded-full transition-all duration-300"
              style={{ width: `${getProgressPercentage()}%` }}
            ></div>
          </div>
        </div>

        {/* Participants */}
        <div className="mb-4">
          <div className="flex items-center justify-between text-sm text-gray-600">
            <span className="font-medium">Participants:</span>
            <span className="flex items-center">
              <svg className="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z"
                />
              </svg>
              {race.participants.length}
            </span>
          </div>
        </div>

        {/* Timestamps */}
        <div className="text-xs text-gray-500 mb-4">
          <p>Created: {formatDate(race.created_at)}</p>
          {race.updated_at !== race.created_at && <p>Updated: {formatDate(race.updated_at)}</p>}
        </div>

        {/* Actions */}
        <div className="flex space-x-2">
          <button
            onClick={() => onAction(race.uuid, 'view')}
            className="flex-1 inline-flex justify-center items-center px-3 py-2 border border-gray-300 shadow-sm text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
          >
            <svg className="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
              />
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z"
              />
            </svg>
            View
          </button>

          {race.status === 'Waiting' && race.participants.length > 0 && (
            <button
              onClick={() => onAction(race.uuid, 'start')}
              className="flex-1 inline-flex justify-center items-center px-3 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-green-600 hover:bg-green-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-green-500"
            >
              <svg className="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M14.828 14.828a4 4 0 01-5.656 0M9 10h1.586a1 1 0 01.707.293l2.414 2.414a1 1 0 00.707.293H15M9 10V9a2 2 0 012-2h2a2 2 0 012 2v1M9 10v5a2 2 0 002 2h2a2 2 0 002-2v-5"
                />
              </svg>
              Start
            </button>
          )}
        </div>
      </div>
    </div>
  );
};

export default RaceCard;
