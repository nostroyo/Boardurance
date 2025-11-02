import React, { useRef, useState } from 'react';

interface Sector {
  id: number;
  name: string;
  min_value: number;
  max_value: number;
  slot_capacity: number | null;
  sector_type: 'Start' | 'Straight' | 'Curve' | 'Finish';
}

interface JSONUploaderProps {
  onTrackLoad: (sectors: Sector[]) => void;
  onError: (error: string) => void;
}

interface TrackJSON {
  sectors: Sector[];
}

const JSONUploader: React.FC<JSONUploaderProps> = ({ onTrackLoad, onError }) => {
  const fileInputRef = useRef<HTMLInputElement>(null);
  const [isDragOver, setIsDragOver] = useState(false);
  const [isProcessing, setIsProcessing] = useState(false);

  const validateTrackJSON = (data: any): Sector[] => {
    // Check if data has sectors array
    if (!data || !Array.isArray(data.sectors)) {
      throw new Error('JSON must contain a "sectors" array');
    }

    const sectors = data.sectors;

    if (sectors.length < 2) {
      throw new Error('Track must have at least 2 sectors');
    }

    // Validate each sector
    const validatedSectors: Sector[] = sectors.map((sector: any, index: number) => {
      // Required fields validation
      if (typeof sector.id !== 'number') {
        throw new Error(`Sector ${index + 1}: "id" must be a number`);
      }
      if (typeof sector.name !== 'string' || !sector.name.trim()) {
        throw new Error(`Sector ${index + 1}: "name" must be a non-empty string`);
      }
      if (typeof sector.min_value !== 'number' || sector.min_value < 0) {
        throw new Error(`Sector ${index + 1}: "min_value" must be a non-negative number`);
      }
      if (typeof sector.max_value !== 'number' || sector.max_value <= sector.min_value) {
        throw new Error(`Sector ${index + 1}: "max_value" must be greater than min_value`);
      }
      if (sector.slot_capacity !== null && (typeof sector.slot_capacity !== 'number' || sector.slot_capacity < 1)) {
        throw new Error(`Sector ${index + 1}: "slot_capacity" must be null or a positive number`);
      }
      if (!['Start', 'Straight', 'Curve', 'Finish'].includes(sector.sector_type)) {
        throw new Error(`Sector ${index + 1}: "sector_type" must be one of: Start, Straight, Curve, Finish`);
      }

      return {
        id: sector.id,
        name: sector.name.trim(),
        min_value: sector.min_value,
        max_value: sector.max_value,
        slot_capacity: sector.slot_capacity,
        sector_type: sector.sector_type
      };
    });

    // Validate sector order and types
    if (validatedSectors[0].sector_type !== 'Start') {
      throw new Error('First sector must be of type "Start"');
    }

    const lastSector = validatedSectors[validatedSectors.length - 1];
    if (lastSector.sector_type !== 'Finish') {
      throw new Error('Last sector must be of type "Finish"');
    }

    // Validate Start and Finish sectors have unlimited capacity
    if (validatedSectors[0].slot_capacity !== null) {
      throw new Error('Start sector must have unlimited capacity (slot_capacity: null)');
    }
    if (lastSector.slot_capacity !== null) {
      throw new Error('Finish sector must have unlimited capacity (slot_capacity: null)');
    }

    // Validate unique IDs
    const ids = validatedSectors.map(s => s.id);
    const uniqueIds = new Set(ids);
    if (ids.length !== uniqueIds.size) {
      throw new Error('All sector IDs must be unique');
    }

    return validatedSectors;
  };

  const processFile = async (file: File) => {
    setIsProcessing(true);
    
    try {
      // Check file type
      if (!file.name.toLowerCase().endsWith('.json')) {
        throw new Error('Please select a JSON file');
      }

      // Check file size (max 1MB)
      if (file.size > 1024 * 1024) {
        throw new Error('File size must be less than 1MB');
      }

      // Read file content
      const text = await file.text();
      
      // Parse JSON
      let jsonData: TrackJSON;
      try {
        jsonData = JSON.parse(text);
      } catch (parseError) {
        throw new Error('Invalid JSON format');
      }

      // Validate track structure
      const validatedSectors = validateTrackJSON(jsonData);

      // Success - call onTrackLoad
      onTrackLoad(validatedSectors);
      
    } catch (error) {
      onError(error instanceof Error ? error.message : 'Failed to process file');
    } finally {
      setIsProcessing(false);
    }
  };

  const handleFileSelect = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (file) {
      processFile(file);
    }
  };

  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragOver(true);
  };

  const handleDragLeave = (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragOver(false);
  };

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragOver(false);
    
    const file = e.dataTransfer.files[0];
    if (file) {
      processFile(file);
    }
  };

  const handleClick = () => {
    fileInputRef.current?.click();
  };

  const generateSampleJSON = () => {
    const sampleTrack = {
      sectors: [
        {
          id: 0,
          name: "Start Line",
          min_value: 0,
          max_value: 10,
          slot_capacity: null,
          sector_type: "Start"
        },
        {
          id: 1,
          name: "Casino Corner",
          min_value: 8,
          max_value: 15,
          slot_capacity: 3,
          sector_type: "Curve"
        },
        {
          id: 2,
          name: "Tunnel Straight",
          min_value: 12,
          max_value: 20,
          slot_capacity: 2,
          sector_type: "Straight"
        },
        {
          id: 3,
          name: "Finish Line",
          min_value: 18,
          max_value: 25,
          slot_capacity: null,
          sector_type: "Finish"
        }
      ]
    };

    const blob = new Blob([JSON.stringify(sampleTrack, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'sample-track.json';
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  };

  return (
    <div className="space-y-4">
      {/* File Upload Area */}
      <div
        className={`relative border-2 border-dashed rounded-lg p-6 transition-colors ${
          isDragOver
            ? 'border-blue-400 bg-blue-50'
            : 'border-gray-300 hover:border-gray-400'
        } ${isProcessing ? 'opacity-50 pointer-events-none' : 'cursor-pointer'}`}
        onDragOver={handleDragOver}
        onDragLeave={handleDragLeave}
        onDrop={handleDrop}
        onClick={handleClick}
      >
        <input
          ref={fileInputRef}
          type="file"
          accept=".json"
          onChange={handleFileSelect}
          className="hidden"
        />
        
        <div className="text-center">
          {isProcessing ? (
            <div className="flex flex-col items-center">
              <svg className="animate-spin h-8 w-8 text-blue-500 mb-2" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
                <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
              </svg>
              <p className="text-sm text-gray-600">Processing track file...</p>
            </div>
          ) : (
            <div>
              <svg className="mx-auto h-12 w-12 text-gray-400" stroke="currentColor" fill="none" viewBox="0 0 48 48">
                <path d="M28 8H12a4 4 0 00-4 4v20m32-12v8m0 0v8a4 4 0 01-4 4H12a4 4 0 01-4-4v-4m32-4l-3.172-3.172a4 4 0 00-5.656 0L28 28M8 32l9.172-9.172a4 4 0 015.656 0L28 28m0 0l4 4m4-24h8m-4-4v8m-12 4h.02" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round" />
              </svg>
              <div className="mt-4">
                <p className="text-sm text-gray-600">
                  <span className="font-medium">Click to upload</span> or drag and drop your track JSON file
                </p>
                <p className="text-xs text-gray-500 mt-1">JSON files only, max 1MB</p>
              </div>
            </div>
          )}
        </div>
      </div>

      {/* Help Section */}
      <div className="bg-gray-50 rounded-lg p-4">
        <h5 className="text-sm font-medium text-gray-900 mb-2">Track JSON Format</h5>
        <div className="text-xs text-gray-600 space-y-1">
          <p>• Must contain a "sectors" array with at least 2 sectors</p>
          <p>• First sector must be type "Start" with unlimited capacity</p>
          <p>• Last sector must be type "Finish" with unlimited capacity</p>
          <p>• Each sector needs: id, name, min_value, max_value, slot_capacity, sector_type</p>
          <p>• Sector types: "Start", "Straight", "Curve", "Finish"</p>
        </div>
        
        <div className="mt-3">
          <button
            onClick={generateSampleJSON}
            className="inline-flex items-center px-3 py-1 border border-gray-300 shadow-sm text-xs font-medium rounded text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
          >
            <svg className="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 10v6m0 0l-3-3m3 3l3-3m2 8H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
            </svg>
            Download Sample JSON
          </button>
        </div>
      </div>
    </div>
  );
};

export default JSONUploader;