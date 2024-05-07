export function create(timeout, element, callback) {
  const state = {}
  state.width = null
  state.height = null
  state.timeout = null
  state.stopped = false
  state.observer = new ResizeObserver(entries => {
    const rect = entries[0].contentRect
    const width = rect.width
    const height = rect.height

    if (width === state.width && height === state.height) {
      return;
    }

    state.width = width
    state.height = height

    if (state.stopped) {
      console.error("Whew i caught a `closure invoked recursively or destroyed already`, but why the fuck did ResizeObserver call this callback?")
      return;
    }

    if (state.timeout !== null) {
      clearTimeout(state.timeout)
      state.timeout = null
    }
    
    state.timeout = setTimeout(() => {
      if (state.stopped) {
        console.error("Whew i caught a `closure invoked recursively or destroyed already`, but why the fuck did the timeout not clear?")
        return;
      } 
      callback(width, height)
    }, timeout)
  })

  state.observer.observe(element)
  return state
}

export function destroy(state) {
  state.stopped = true
  state.observer.disconnect()
  
  if (state.timeout !== null) {
    clearTimeout(state.timeout)
    state.timeout = null
  }
}