import React from 'react';

export default function StatusIndicator({ connected }) {
  return (
    <div className="flex items-center gap-2 px-4 py-2 bg-gray-800/50 border-b border-gray-700">
      <div className="flex items-center gap-2">
        <div className={`w-2 h-2 rounded-full ${connected ? 'bg-green-500 animate-pulse' : 'bg-red-500'}`} />
        <span className="text-sm text-gray-300">
          {connected ? 'Connected to Swarm' : 'Disconnected'}
        </span>
      </div>
      <div className="ml-auto text-xs text-gray-500">
        Axon Cluster Web UI
      </div>
    </div>
  );
}
