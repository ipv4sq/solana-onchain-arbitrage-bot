import { NextResponse } from 'next/server'
import axios from 'axios'

const RUST_SERVER_URL = process.env.RUST_SERVER_URL || 'http://localhost:8080'

export async function GET() {
  try {
    const response = await axios.get(`${RUST_SERVER_URL}/config`)
    return NextResponse.json({ 
      success: true, 
      config: response.data 
    })
  } catch (error) {
    console.error('Failed to fetch config from Rust server:', error)
    return NextResponse.json(
      { success: false, error: 'Failed to fetch configuration' },
      { status: 500 }
    )
  }
}

export async function POST(request: Request) {
  try {
    const body = await request.json()
    // TODO: Implement config update endpoint in Rust server
    return NextResponse.json({ 
      success: true, 
      message: 'Configuration update endpoint not yet implemented' 
    })
  } catch (error) {
    return NextResponse.json(
      { success: false, error: 'Failed to update configuration' },
      { status: 500 }
    )
  }
}