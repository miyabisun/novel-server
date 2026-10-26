import { mount } from 'svelte'
import 'normalize.css'
import './global.sass'
import App from './App.svelte'
import { applyTheme, loadTheme } from './lib/theme.js'
import { followThemeColor } from './lib/theme-color.js'

applyTheme(loadTheme())
followThemeColor()

mount(App, { target: document.getElementById('app') })
