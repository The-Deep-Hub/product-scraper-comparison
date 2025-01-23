import React from "react";

type MessageDialogProps = {
  message: string;
  type: "loading" | "success" | "error";
  onClose: () => void;
};

const MessageDialog: React.FC<MessageDialogProps> = ({ message, type, onClose }) => {
  // Translate default messages if needed
  const getTranslatedMessage = (originalMessage: string): string => {
    const translations: { [key: string]: string } = {
      "Retrieving search results, please wait...": "Buscando resultados, por favor espere...",
      "Error retrieving products. Please try again.": "Error al recuperar los productos. Por favor, inténtelo de nuevo.",
      "A search is already in progress. Please wait for it to finish.": "Ya hay una búsqueda en curso. Por favor, espere a que termine.",
      "Please enter a search query.": "Por favor, introduzca un término de búsqueda.",
      "Invalid response from server": "Respuesta inválida del servidor",
      "Failed to initiate search": "Error al iniciar la búsqueda",
      "Search time completed. Displaying found results.": "Tiempo de búsqueda completado. Mostrando resultados encontrados.",
      "Failed to retrieve search results": "Error al recuperar los resultados de la búsqueda",
      "Error retrieving search results.": "Error al recuperar los resultados de la búsqueda."
    };

    return translations[originalMessage] || originalMessage;
  };

  const getMessageStyles = () => {
    switch (type) {
      case "loading":
        return "bg-blue-500";
      case "success":
        return "bg-green-500";
      case "error":
        return "bg-red-500";
      default:
        return "bg-gray-500";
    }
  };

  return (
    <div className="fixed top-0 left-0 w-full h-full flex items-center justify-center bg-black bg-opacity-50">
      <div className={`relative p-6 rounded-md shadow-lg text-white ${getMessageStyles()} w-3/4 md:w-1/3`}>
        <button 
          className="absolute top-2 right-2 text-white text-lg" 
          onClick={onClose}
          aria-label="Cerrar mensaje"
        >
          ✖
        </button>
        <p className="text-lg font-semibold">{getTranslatedMessage(message)}</p>
      </div>
    </div>
  );
};

export default MessageDialog;
