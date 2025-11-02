import React, { useState } from 'react';
import { useAuthContext } from '../contexts/AuthContext';
import RaceCreator from './admin/RaceCreator';
import RaceDashboard from './admin/RaceDashboard';

type AdminView = 'dashboard' | 'create-race';

const AdminDashboard: React.FC = () => {
  const { user, logout } = useAuthContext();
  const [currentView, setCurrentView] = useState<AdminView>('dashboard');

  const handleLogout = async () => {
    await logout();
  };

  const renderContent = () => {
    switch (currentView) {
      case 'create-race':
        return <RaceCreator onRaceCreated={() => setCurrentView('dashboard')} />;
      case 'dashboard':
      default:
        return <RaceDashboard />;
    }
  };

  return (
    <div className="min-h-screen bg-gray-50">
      {/* Navigation Header */}
      <nav className="bg-white shadow-sm border-b border-gray-200">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between h-16">
            <div className="flex">
              <div className="flex-shrink-0 flex items-center">
                <h1 className="text-xl font-bold text-gray-900">Admin Dashboard</h1>
              </div>
              <div className="hidden sm:ml-6 sm:flex sm:space-x-8">
                <button
                  onClick={() => setCurrentView('dashboard')}
                  className={`${
                    currentView === 'dashboard'
                      ? 'border-blue-500 text-gray-900'
                      : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
                  } whitespace-nowrap py-2 px-1 border-b-2 font-medium text-sm`}
                >
                  Race Management
                </button>
                <button
                  onClick={() => setCurrentView('create-race')}
                  className={`${
                    currentView === 'create-race'
                      ? 'border-blue-500 text-gray-900'
                      : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
                  } whitespace-nowrap py-2 px-1 border-b-2 font-medium text-sm`}
                >
                  Create Race
                </button>
              </div>
            </div>
            <div className="flex items-center">
              <div className="flex-shrink-0">
                <span className="text-sm text-gray-500 mr-4">
                  Welcome, {user?.email} (Admin)
                </span>
                <button
                  onClick={handleLogout}
                  className="bg-white py-2 px-3 border border-gray-300 rounded-md shadow-sm text-sm leading-4 font-medium text-gray-700 hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
                >
                  Logout
                </button>
              </div>
            </div>
          </div>
        </div>
      </nav>

      {/* Mobile Navigation */}
      <div className="sm:hidden">
        <div className="pt-2 pb-3 space-y-1 bg-white border-b border-gray-200">
          <button
            onClick={() => setCurrentView('dashboard')}
            className={`${
              currentView === 'dashboard'
                ? 'bg-blue-50 border-blue-500 text-blue-700'
                : 'border-transparent text-gray-600 hover:bg-gray-50 hover:border-gray-300 hover:text-gray-800'
            } block pl-3 pr-4 py-2 border-l-4 text-base font-medium w-full text-left`}
          >
            Race Management
          </button>
          <button
            onClick={() => setCurrentView('create-race')}
            className={`${
              currentView === 'create-race'
                ? 'bg-blue-50 border-blue-500 text-blue-700'
                : 'border-transparent text-gray-600 hover:bg-gray-50 hover:border-gray-300 hover:text-gray-800'
            } block pl-3 pr-4 py-2 border-l-4 text-base font-medium w-full text-left`}
          >
            Create Race
          </button>
        </div>
      </div>

      {/* Main Content */}
      <main className="max-w-7xl mx-auto py-6 sm:px-6 lg:px-8">
        <div className="px-4 py-6 sm:px-0">
          {renderContent()}
        </div>
      </main>
    </div>
  );
};

export default AdminDashboard;