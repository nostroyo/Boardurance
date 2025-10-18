import { Link } from 'react-router-dom';

function MainPage() {
  return (
    <div className="min-h-screen bg-gradient-to-br from-blue-900 to-purple-900 flex items-center justify-center">
      <div className="bg-white rounded-lg shadow-xl p-8 max-w-md w-full mx-4">
        <div className="text-center">
          <h1 className="text-3xl font-bold text-gray-800 mb-2">
            Web3 Racing Game
          </h1>
          <p className="text-gray-600 mb-8">
            Welcome to the ultimate motorsport management experience on the blockchain
          </p>
          
          <div className="space-y-4">
            <Link
              to="/register"
              className="w-full bg-blue-600 hover:bg-blue-700 text-white font-semibold py-3 px-6 rounded-lg transition duration-200 block text-center"
            >
              Create Account
            </Link>
            
            <Link
              to="/login"
              className="w-full bg-gray-600 hover:bg-gray-700 text-white font-semibold py-3 px-6 rounded-lg transition duration-200 block text-center"
            >
              Login
            </Link>
          </div>
          
          <div className="mt-8 text-sm text-gray-500">
            <p>Join the race. Own your assets. Win on the blockchain.</p>
          </div>
        </div>
      </div>
    </div>
  );
}

export default MainPage;