import React from "react";

type MessageDialogProps = {
  message: string;
  type: "loading" | "success" | "error";
  onClose: () => void;
};

const MessageDialog: React.FC<MessageDialogProps> = ({ message, type, onClose }) => {
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
        <button className="absolute top-2 right-2 text-white text-lg" onClick={onClose}>
          ✖
        </button>
        <p className="text-lg font-semibold">{message}</p>
      </div>
    </div>
  );
};

export default MessageDialog;
