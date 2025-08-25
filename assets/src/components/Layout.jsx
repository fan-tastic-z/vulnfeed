import Navigation from './Navigation'

const Layout = ({ children }) => {
  return (
    <div className="min-h-screen flex flex-col bg-gray-50">
      <Navigation />
      <div className="flex-grow py-10">
        <main>
          <div className="max-w-7xl mx-auto sm:px-6 lg:px-8">
            {children}
          </div>
        </main>
      </div>
      <footer className="bg-white border-t border-gray-200">
        <div className="max-w-7xl mx-auto py-6 px-4 sm:px-6 lg:px-8">
          <p className="text-center text-sm text-gray-500">
            © {new Date().getFullYear()} VulnFeed. All rights reserved.
          </p>
        </div>
      </footer>
    </div>
  )
}

export default Layout
