import { mount } from 'svelte'
import 'normalize.css'
import './global.sass'
import App from './App.svelte'
import { applyTheme, loadTheme } from './lib/theme.js'

applyTheme(loadTheme())

mount(App, { target: document.getElementById('app') })
