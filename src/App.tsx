'use client'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { open } from '@tauri-apps/api/dialog'
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/tauri'
import { downloadDir } from '@tauri-apps/api/path'
import './index.css'
import { useEffect } from 'react';

interface retorno {
  message: string
}

function App() {
    useEffect(() => {
      if (window.__TAURI__ !== undefined) {
        const unlisten = listen<retorno>('download_completed', ({payload}) => {
          console.log(payload.message)
        })
  
        return () => {
          unlisten.then(f => f())
        }
      }
    }, [])
  let link = ''
  const handleDownloadClick = async () => {
    const downloadDirPath = await downloadDir()
    const savePath = await open({
      defaultPath: downloadDirPath,
      multiple: false,
      directory: true,
      title: 'Select output folder',
    })
    if (typeof savePath !== 'string') {
      return
    }
    console.log(savePath)
    invoke<string>('download_handler', {
      url: link,
      output: savePath,
    })
      .then((result) => console.log(result))
      .catch(console.error)
  }

  return (
    <div className="flex w-full max-w-sm items-center space-x-2">
      <div>
        <Input
          type="url"
          placeholder="Youtube link"
          id="url-input"
          onChange={(e) => (link = e.target.value)}
        />
        <Button onClick={handleDownloadClick}>Download</Button>
      </div>
    </div>
  )
}

export default App
