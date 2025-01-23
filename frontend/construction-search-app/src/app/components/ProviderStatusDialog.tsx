import React, { useState } from "react";

type ProviderStatusDialogProps = {
  pendingStores: string[];
  completedStores: string[];
};

const ProviderStatusDialog: React.FC<ProviderStatusDialogProps> = ({ pendingStores, completedStores }) => {
  const [isMinimized, setIsMinimized] = useState(false);

  return (
    <div className="fixed z-10 bottom-4 right-4 bg-white shadow-md rounded-md p-4 w-80 border">
      <div className="flex justify-between">
        <h3 className="font-semibold text-lg">Estado de las tiendas</h3>
        <button onClick={() => setIsMinimized(!isMinimized)} className="text-lg">
          {isMinimized ? "▲" : "▼"}
        </button>
      </div>
      {!isMinimized && (
        <div className="mt-2">
          <ul className="space-y-2">
            {completedStores.map((store) => (
              <li key={store} className="flex items-center">
                <input type="checkbox" checked disabled className="mr-2" />
                <span className="text-green-600">{store} (Completado)</span>
              </li>
            ))}
            {pendingStores.map((store) => (
              <li key={store} className="flex items-center">
                <input type="checkbox" disabled className="mr-2" />
                <span className="text-yellow-600">{store} (Pendiente...)</span>
              </li>
            ))}
          </ul>
        </div>
      )}
    </div>
  );
};

export default ProviderStatusDialog;
