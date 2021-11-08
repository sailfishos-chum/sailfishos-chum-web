'strict'

let repositories  = null
let sfos          = null
let arch          = null
let loading       = null
let chum_link     = null
let gui_link      = null
let no_chum_link  = null
let no_gui_link   = null

window.onload = async () => {
  sfos          = document.getElementById('sfos')
  arch          = document.getElementById('arch')
  loading       = document.getElementById('loading')
  chum_link     = document.getElementById('chum-link')
  gui_link      = document.getElementById('chum-gui-link')
  no_chum_link  = document.getElementById('no-chum-link')
  no_gui_link   = document.getElementById('no-chum-gui-link')

  await fetch_versions()

  loading.hidden = true
  document.getElementById('form').hidden = false
}

function option(v) {
  const o = document.createElement('option')
  o.value = v
  o.text  = v
  return o
}

async function fetch_versions() {
  const repos_reply = await fetch('/.netlify/functions/lambda/repositories')
  const maybe_repos = await repos_reply.json()

  if (maybe_repos.error) {
    alert(
`Failed to fetch available repositories

Server reply: ${maybe_repos.error}`
    )
    return
  }

  repositories = maybe_repos.repositories

  repositories.forEach(([v]) => sfos.appendChild(option(v)))
  sfos.selectedIndex = 0
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

function hide_link_elements() {
  chum_link.hidden    = true
  gui_link.hidden     = true
  no_chum_link.hidden = true
  no_gui_link.hidden  = true
}

async function onsfos() {
  hide_link_elements()

  let arch_value = arch.value

  while (arch.lastChild.value !== 'none') {
    arch.removeChild(arch.lastChild)
  }

  const archs = repositories[sfos.selectedIndex - 1][1]
  archs.forEach(a => arch.appendChild(option(a)))
  document.getElementById('arch-container').hidden = false

  if (archs.includes(arch_value)) {
    arch.value = arch_value
    onarch()
  } else {
    arch.selectedIndex = 0
  }
}

async function onarch() {
  hide_link_elements()

  const sfos_value = sfos.value
  const arch_value = arch.value

  if (sfos_value === 'none' || arch_value === 'none') {
    return
  }

  loading.hidden = false
  const res = await fetch(`/.netlify/functions/lambda/packages/${sfos.value}_${arch.value}`)
  const {chum, gui} = await res.json()
  loading.hidden = true
  set_link(chum, chum_link, no_chum_link)
  set_link(gui,  gui_link,  no_gui_link)
}
