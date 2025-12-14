import React, { useState, useEffect, useRef } from 'react';
import StatusIndicator from './StatusIndicator';
import MessageBubble from './MessageBubble';
import InputArea from './InputArea';

const API_BASE = 'http://localhost:3000';

export default function ChatWindow() {
  const [messages, setMessages] = useState([]);
  const [connected, setConnected] = useState(false);
  const [loading, setLoading] = useState(false);
  const messagesEndRef = useRef(null);

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  };

  useEffect(() => {
    scrollToBottom();
  }, [messages]);

  // Check connection status
  useEffect(() => {
    const checkConnection = async () => {
      try {
        const response = await fetch(`${API_BASE}/api/health`);
        setConnected(response.ok);
      } catch {
        setConnected(false);
      }
    };

    checkConnection();
    const interval = setInterval(checkConnection, 5000);
    return () => clearInterval(interval);
  }, []);

  const handleSend = async (text) => {
    const userMessage = {
      text,
      isUser: true,
      timestamp: Date.now(),
    };

    const thinkingMessage = {
      text: '',
      isUser: false,
      isThinking: true,
      timestamp: Date.now(),
    };

    setMessages((prev) => [...prev, userMessage, thinkingMessage]);
    setLoading(true);

    try {
      const response = await fetch(`${API_BASE}/api/ask`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ prompt: text }),
      });

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      const data = await response.json();

      setMessages((prev) => {
        const filtered = prev.filter((msg) => !msg.isThinking);
        return [
          ...filtered,
          {
            text: data.answer || data.response || 'No response received',
            isUser: false,
            timestamp: Date.now(),
          },
        ];
      });
    } catch (error) {
      console.error('Error sending message:', error);
      setMessages((prev) => {
        const filtered = prev.filter((msg) => !msg.isThinking);
        return [
          ...filtered,
          {
            text: `Error: ${error.message}. Make sure the Rust backend is running.`,
            isUser: false,
            timestamp: Date.now(),
          },
        ];
      });
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="flex flex-col h-screen bg-gray-900">
      <StatusIndicator connected={connected} />
      
      <div className="flex-1 overflow-y-auto p-4 space-y-2">
        {messages.length === 0 && (
          <div className="flex items-center justify-center h-full">
            <div className="text-center text-gray-500">
              <div className="text-6xl mb-4">🧠⚡</div>
              <h2 className="text-2xl font-bold mb-2">Axon Cluster</h2>
              <p>Start a conversation with your AI Leader node</p>
            </div>
          </div>
        )}
        {messages.map((msg, idx) => (
          <MessageBubble key={idx} message={msg} isUser={msg.isUser} />
        ))}
        <div ref={messagesEndRef} />
      </div>

      <InputArea onSend={handleSend} disabled={loading || !connected} />
    </div>
  );
}
