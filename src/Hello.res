// src/Hello.res
// A simple ReScript 12 component

// Import React bindings
@react.component
let make = () => {
  // Log to console when component is rendered
  React.useEffect(() => {
    Js.log("Hello from Rumi!")
    None // No cleanup needed for this effect
  }, [])

  <div> {React.string("Hello from Rumi!")} </div>
}

// Export the component
// This makes it available for import in other modules
// For example, to be used in src/Main.res or src/App.res
// If this were part of a larger app, you might register it here
// or import it elsewhere. For this standalone test, we ensure it's defined.
// The export is implicit when a file defines a top-level component like this
// and is intended to be imported.
