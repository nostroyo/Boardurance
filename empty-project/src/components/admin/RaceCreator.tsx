import React, { useState } from 'react';
import JSONUploader from './JSONUploader';
import { raceAPI } from '../../utils/raceAPI';

interface RaceCreatorProps {
  onRaceCreated: () => void;
}

interface Sector {
  id: number;
  name: string;
  min_value: number;
  max_value: number;
  slot_capacity: number | null;
  sector_type: 'Start' | 'Straight' | 'Curve' | 'Finish';
}

interface RaceFormData {
  raceName: string;
  trackName: string;
  totalLaps: number;
  sectors: Sector[];
}

const RaceCreator: React.FC<RaceCreatorProps> = ({ onRaceCreated }) => {
  const [formData, setFormData] = useState<RaceFormData>({
    raceName: '',
    trackName: '',
    totalLaps: 3,
    sectors: []
  });
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const { name, value } = e.target;
    setFormData(prev => ({
      ...prev,
      [name]: name === 'totalLaps' ? parseInt(value) || 1 : value
    }));
  };

  const handleTrackLoad = (sectors: Sector[]) => {
    setFormData(prev => ({ ...prev, sectors }));
    setError(null);
  };

  const handleTrackError = (errorMessage: string) => {
    setError(errorMessage);
    setSuccess(null);
  };

  const validateForm = (): string | null => {
    if (!formData.raceName.trim()) {
      return 'Race name is required';
    }
    if (!formData.trackName.trim()) {
      return 'Track name is required';
    }
    if (formData.totalLaps < 1 || formData.totalLaps > 100) {
      return 'Total laps must be between 1 and 100';
    }
    if (formData.sectors.length < 2) {
      return 'Track must have at least 2 sectors (Start and Finish)';
    }
    
    // Validate first sector is Start type
    if (formData.sectors[0].sector_type !== 'Start') {
      return 'First sector must be of type "Start"';
    }
    
    // Validate last sector is Finish type
    const lastSector = formData.sectors[formData.sectors.length - 1];
    if (lastSector.sector_type !== 'Finish') {
      return 'Last sector must be of type "Finish"';
    }

    return null;
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    
    const validationError = validateForm();
    if (validationError) {
      setError(validationError);
      return;
    }

    setIsSubmitting(true);
    setError(null);
    setSuccess(null);

    try {
      const result = await raceAPI.createRace({
        name: formData.raceName,
        track_name: formData.trackName,
        total_laps: formData.totalLaps,
        sectors: formData.sectors.map(sector => ({
          id: sector.id,
          name: sector.name,
          min_value: sector.min_value,
          max_value: sector.max_value,
          slot_capacity: sector.slot_capacity,
          sector_type: sector.sector_type
        }))
      });

      if (result.success) {
        setSuccess(`Race "${formData.raceName}" created successfully!`);
        // Reset form
        setFormData({
          raceName: '',
          trackName: '',
          totalLaps: 3,
          sectors: []
        });
        // Notify parent component
        setTimeout(() => {
          onRaceCreated();
        }, 2000);
      } else {
        setError(result.error || 'Failed to create race');
      }
    } catch (error) {
      setError('Network error. Please try again.');
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <div className="max-w-4xl mx-auto">
      <div className="bg-white shadow rounded-lg">
        <div className="px-4 py-5 sm:p-6">
          <h3 className="text-lg leading-6 font-medium text-gray-900 mb-6">
            Create New Race
          </h3>

          {error && (
            <div className="mb-4 bg-red-50 border border-red-200 rounded-md p-4">
              <div className="flex">
                <div className="flex-shrink-0">
                  <svg className="h-5 w-5 text-red-400" viewBox="0 0 20 20" fill="currentColor">
                    <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clipRule="evenodd" />
                  </svg>
                </div>
                <div className="ml-3">
                  <p className="text-sm text-red-800">{error}</p>
                </div>
              </div>
            </div>
          )}

          {success && (
            <div className="mb-4 bg-green-50 border border-green-200 rounded-md p-4">
              <div className="flex">
                <div className="flex-shrink-0">
                  <svg className="h-5 w-5 text-green-400" viewBox="0 0 20 20" fill="currentColor">
                    <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clipRule="evenodd" />
                  </svg>
                </div>
                <div className="ml-3">
                  <p className="text-sm text-green-800">{success}</p>
                </div>
              </div>
            </div>
          )}

          <form onSubmit={handleSubmit} className="space-y-6">
            {/* Race Basic Information */}
            <div className="grid grid-cols-1 gap-6 sm:grid-cols-2">
              <div>
                <label htmlFor="raceName" className="block text-sm font-medium text-gray-700">
                  Race Name
                </label>
                <input
                  type="text"
                  name="raceName"
                  id="raceName"
                  value={formData.raceName}
                  onChange={handleInputChange}
                  className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
                  placeholder="Enter race name"
                  required
                />
              </div>

              <div>
                <label htmlFor="trackName" className="block text-sm font-medium text-gray-700">
                  Track Name
                </label>
                <input
                  type="text"
                  name="trackName"
                  id="trackName"
                  value={formData.trackName}
                  onChange={handleInputChange}
                  className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
                  placeholder="Enter track name"
                  required
                />
              </div>

              <div>
                <label htmlFor="totalLaps" className="block text-sm font-medium text-gray-700">
                  Total Laps
                </label>
                <input
                  type="number"
                  name="totalLaps"
                  id="totalLaps"
                  min="1"
                  max="100"
                  value={formData.totalLaps}
                  onChange={handleInputChange}
                  className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
                  required
                />
              </div>
            </div>

            {/* Track Configuration */}
            <div>
              <h4 className="text-md font-medium text-gray-900 mb-4">Track Configuration</h4>
              <JSONUploader onTrackLoad={handleTrackLoad} onError={handleTrackError} />
            </div>

            {/* Track Preview */}
            {formData.sectors.length > 0 && (
              <div>
                <h4 className="text-md font-medium text-gray-900 mb-4">Track Preview</h4>
                <div className="bg-gray-50 rounded-lg p-4">
                  <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3">
                    {formData.sectors.map((sector, index) => (
                      <div key={sector.id} className="bg-white rounded-lg p-3 shadow-sm border">
                        <div className="flex items-center justify-between mb-2">
                          <span className="text-sm font-medium text-gray-900">
                            {index + 1}. {sector.name}
                          </span>
                          <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
                            sector.sector_type === 'Start' ? 'bg-green-100 text-green-800' :
                            sector.sector_type === 'Finish' ? 'bg-red-100 text-red-800' :
                            sector.sector_type === 'Straight' ? 'bg-blue-100 text-blue-800' :
                            'bg-yellow-100 text-yellow-800'
                          }`}>
                            {sector.sector_type}
                          </span>
                        </div>
                        <div className="text-xs text-gray-500">
                          <div>Range: {sector.min_value} - {sector.max_value}</div>
                          <div>Capacity: {sector.slot_capacity || 'Unlimited'}</div>
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              </div>
            )}

            {/* Submit Button */}
            <div className="flex justify-end">
              <button
                type="submit"
                disabled={isSubmitting || formData.sectors.length === 0}
                className="inline-flex justify-center py-2 px-4 border border-transparent shadow-sm text-sm font-medium rounded-md text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {isSubmitting ? (
                  <>
                    <svg className="animate-spin -ml-1 mr-3 h-5 w-5 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                      <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
                      <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                    </svg>
                    Creating Race...
                  </>
                ) : (
                  'Create Race'
                )}
              </button>
            </div>
          </form>
        </div>
      </div>
    </div>
  );
};

export default RaceCreator;