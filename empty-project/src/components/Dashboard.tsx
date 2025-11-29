import { Link } from 'react-router-dom';
import { useAuthContext } from '../contexts/AuthContext';

function Dashboard() {
  const { user, logout } = useAuthContext();
  return (
    <div className="min-h-screen bg-gradient-to-br from-blue-900 to-purple-900 flex items-center justify-center">
      <div className="bg-white rounded-lg shadow-xl p-8 max-w-2xl w-full mx-4">
        <div className="text-center mb-8">
          <div className="mb-4">
            <div className="w-16 h-16 bg-green-100 rounded-full flex items-center justify-center mx-auto">
              <svg className="w-8 h-8 text-green-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
              </svg>
            </div>
          </div>
          <h1 className="text-3xl font-bold text-gray-800 mb-2">
            Welcome back, {user?.team_name || 'Racer'}!
          </h1>
          <p className="text-gray-600 mb-2">
            Ready to dominate the track? Your racing empire awaits!
          </p>
          <p className="text-sm text-gray-500 mb-6">
            Logged in as: {user?.email}
          </p>
        </div>
        
        <div className="grid md:grid-cols-2 gap-6 mb-8">
          <div className="bg-gray-50 p-6 rounded-lg">
            <h3 className="text-lg font-semibold text-gray-800 mb-2">Next Steps</h3>
            <ul className="text-sm text-gray-600 space-y-2">
              <li>• Connect your Solana wallet</li>
              <li>• Acquire your first racing cars</li>
              <li>• Hire skilled pilots</li>
              <li>• Join your first race</li>
            </ul>
          </div>
          
          <div className="bg-gray-50 p-6 rounded-lg">
            <h3 className="text-lg font-semibold text-gray-800 mb-2">Game Features</h3>
            <ul className="text-sm text-gray-600 space-y-2">
              <li>• NFT-based cars and pilots</li>
              <li>• Blockchain-verified races</li>
              <li>• Team management</li>
              <li>• Competitive tournaments</li>
            </ul>
          </div>
        </div>
        
        <div className="flex flex-col sm:flex-row gap-4 justify-center">
          <button className="bg-blue-600 hover:bg-blue-700 text-white font-semibold py-3 px-6 rounded-lg transition duration-200">
            Connect Wallet
          </button>
          <Link
            to="/team"
            className="bg-purple-600 hover:bg-purple-700 text-white font-semibold py-3 px-6 rounded-lg transition duration-200 text-center"
          >
            Manage Team
          </Link>
          <Link
            to="/game"
            className="bg-green-600 hover:bg-green-700 text-white font-semibold py-3 px-6 rounded-lg transition duration-200 text-center"
          >
            Start Racing
          </Link>
        </div>
        
        <div className="mt-6 text-center">
          <button
            onClick={async () => {
              await logout();
            }}
            className="text-sm text-gray-500 hover:text-gray-700 underline"
          >
            Logout
          </button>
        </div>
      </div>
    </div>
  );
}

export default Dashboard;