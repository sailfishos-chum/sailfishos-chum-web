'strict'

let sfos          = null
let arch          = null
let aarch64       = null
let loading       = null
let chum_link     = null
let gui_link      = null
let no_chum_link  = null
let no_gui_link   = null

window.onload = async () => {
  sfos          = document.getElementById('sfos')
  arch          = document.getElementById('arch')
  aarch64       = document.getElementById('aarch64')
  loading       = document.getElementById('loading')
  chum_link     = document.getElementById('chum-link')
  gui_link      = document.getElementById('chum-gui-link')
  no_chum_link  = document.getElementById('no-chum-link')
  no_gui_link   = document.getElementById('no-chum-gui-link')

  await update()
}

function set_link(url, link, no_link) {
  if (url) {
    link.hidden    = false
    link.href      = url
    link.innerText = url.split('/').pop()
  } else {
    no_link.hidden = false
  }
}

async function update() {
  chum_link.hidden    = true
  gui_link.hidden     = true
  no_chum_link.hidden = true
  no_gui_link.hidden  = true

  const sfos_value = sfos.value
  aarch64.hidden = sfos_value < '4.0.1.48'
  if (aarch64.hidden && arch.value === 'aarch64') {
    arch.value = 'none'
  }
  const arch_value = arch.value
  const show_link  = sfos_value !== 'none' && arch_value !== 'none'
  if (show_link) {
    loading.hidden = false
    const res = await fetch(`/.netlify/functions/lambda/${sfos.value}/${arch.value}`)
    const {chum, gui} = await res.json()
    loading.hidden = true
    set_link(chum, chum_link, no_chum_link)
    set_link(gui,  gui_link,  no_gui_link)
  }
}
