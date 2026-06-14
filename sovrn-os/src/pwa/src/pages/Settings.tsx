import { useState } from 'preact/hooks';
import { authToken } from '../App';

export function Settings() {
  const [activeTab, setActiveTab] = useState('general');

  function handleLogout() {
    authToken.value = null;
    localStorage.removeItem('sovrn_token');
  }

  return (
    <div class="settings">
      <h2>Settings</h2>
      <div class="settings-tabs">
        {['general', 'security', 'network', 'about'].map(tab => (
          <button
            class={`btn btn-ghost settings-tab ${tab === activeTab ? 'settings-tab--active' : ''}`}
            onClick={() => setActiveTab(tab)}
          >
            {tab.charAt(0).toUpperCase() + tab.slice(1)}
          </button>
        ))}
      </div>

      <div class="card settings-content">
        {activeTab === 'general' && (
          <div class="settings-general">
            <h3>General</h3>
            <label class="settings-field">
              <span>Theme</span>
              <select class="input">
                <option value="dark">Dark (Teal)</option>
                <option value="light">Light</option>
              </select>
            </label>
            <label class="settings-field">
              <span>Language</span>
              <select class="input">
                <option value="en">English</option>
                <option value="hi">हिन्दी</option>
              </select>
            </label>
          </div>
        )}

        {activeTab === 'security' && (
          <div class="settings-security">
            <h3>Security & Keys</h3>
            <p class="settings-description">Manage your identity keys and seed phrase.</p>
            <button class="btn btn-secondary">Export Seed Phrase</button>
            <button class="btn btn-secondary">View App Passwords</button>
          </div>
        )}

        {activeTab === 'network' && (
          <div class="settings-network">
            <h3>Network</h3>
            <p class="settings-description">Yggdrasil mesh network settings.</p>
            <label class="settings-field">
              <span>Yggdrasil Address</span>
              <input class="input" readonly value="Not connected" />
            </label>
            <label class="settings-field">
              <span>Connected Peers</span>
              <input class="input" readonly value="0" />
            </label>
          </div>
        )}

        {activeTab === 'about' && (
          <div class="settings-about">
            <h3>About Sovrn OS</h3>
            <p>Version: 0.1.0</p>
            <p>A privacy-first, decentralized operating system.</p>
            <p>Built with ❤ in Jaipur, Rajasthan</p>
          </div>
        )}
      </div>

      <div class="settings-logout">
        <button class="btn btn-secondary" onClick={handleLogout}>Sign Out</button>
      </div>
    </div>
  );
}
