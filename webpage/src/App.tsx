import { useState } from 'react'
import { Routes, Route, useNavigate } from 'react-router-dom'
import './App.css'
import Home from './Home'

function LoginForm() {
  const [username, setUsername] = useState("")
  const [password, setPassword] = useState("")
  const [isSignup, setIsSignup] = useState(false)
  const [message, setMessage] = useState("")
  const navigate = useNavigate()

  const isValidEmail = (email: string): boolean => {
    return email.includes('@') &&
           email.split('@').filter(part => part.length > 0).length === 2 &&
           email.split('@')[1].includes('.') &&
           email.split('@')[1].length > 3 &&
           !email.startsWith('@') &&
           !email.endsWith('@') &&
           email.length > 5
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setMessage("")

    if (isSignup && !isValidEmail(username)) {
      setMessage("Please enter a valid email address")
      return
    }

    try {
      const endpoint = isSignup ? '/signup' : '/login'
      const response = await fetch(`/api${endpoint}`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          username,
          password
        })
      })

      if (response.ok) {
        setMessage(isSignup ? 'Signup successful!' : 'Login successful!')
        if (isSignup) {
          setUsername("")
          setPassword("")
        } else {
          navigate('/home')
        }
      } else {
        if (response.status === 400) {
          setMessage("Invalid email format")
        } else if (response.status === 409) {
          setMessage("User already exists")
        } else if (response.status === 401) {
          setMessage("Invalid credentials")
        } else {
          setMessage(isSignup ? 'Signup failed' : 'Login failed')
        }
      }
    } catch (error) {
      console.error(`Error during ${isSignup ? 'signup' : 'login'}:`, error)
      setMessage("Network error occurred")
    }
  }

  return (
    <>
      <div>
        <h1>Welcome to the Madison Housing Dataset</h1>
        <div>
          <button
            onClick={() => setIsSignup(false)}
            style={{ marginRight: '10px', backgroundColor: !isSignup ? '#007bff' : '#ccc' }}
          >
            Login
          </button>
          <button
            onClick={() => setIsSignup(true)}
            style={{ backgroundColor: isSignup ? '#007bff' : '#ccc' }}
          >
            Sign Up
          </button>
        </div>
        <form onSubmit={handleSubmit}>
          <div>
            <label htmlFor="username">Email:</label>
            <input
              type="email"
              id="username"
              value={username}
              onChange={(e) => setUsername(e.target.value)}
              required
            />
          </div>
          <div>
            <label htmlFor="password">Password:</label>
            <input
              type="password"
              id="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              required
            />
          </div>
          <button type="submit">{isSignup ? 'Sign Up' : 'Login'}</button>
        </form>
        {message && <div style={{ marginTop: '10px', color: message.includes('successful') ? 'green' : 'red' }}>{message}</div>}
      </div>
    </>
  )
}

function App() {
  return (
    <Routes>
      <Route path="/" element={<LoginForm />} />
      <Route path="/home" element={<Home />} />
    </Routes>
  )
}

export default App
